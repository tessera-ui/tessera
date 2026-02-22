//! This module provides the global runtime state management for tessera.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    hash::{Hash, Hasher},
    marker::PhantomData,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use parking_lot::{Mutex, RwLock};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use slotmap::{SlotMap, new_key_type};

use crate::{
    NodeId,
    component_tree::ComponentTree,
    layout::{LayoutSpec, LayoutSpecDyn},
    prop::{ComponentReplayData, ErasedComponentRunner, Prop},
};

thread_local! {
    /// Stack of currently executing component node ids for the current thread.
    static NODE_CONTEXT_STACK: RefCell<Vec<NodeId>> = const { RefCell::new(Vec::new()) };
    /// Control-flow grouping path for the current thread.
    static GROUP_PATH_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    /// Component-instance logic identifier stack (one per component invocation).
    static INSTANCE_LOGIC_ID_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    /// Current execution phase stack for the thread.
    static PHASE_STACK: RefCell<Vec<RuntimePhase>> = const { RefCell::new(Vec::new()) };
    /// Call counter stack: tracks sequential remember calls within each group.
    /// Each entry corresponds to a group depth level.
    static CALL_COUNTER_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };

    /// Call counter stack: tracks sequential context provider calls within each group.
    /// This must not share state with `CALL_COUNTER_STACK`, otherwise `provide_context`
    /// would perturb `remember` slot keys.
    static CONTEXT_CALL_COUNTER_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };

    /// Instance key stack: overrides instance identity inside `key` blocks.
    static INSTANCE_KEY_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };

    /// Call counter stack used for instance identity, reset by `key` blocks.
    static INSTANCE_CALL_COUNTER_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };

    /// Call counter stack used by frame-nanos receivers.
    ///
    /// This must be independent from `CALL_COUNTER_STACK` so frame-receive APIs
    /// never perturb `remember` slot identity.
    static FRAME_RECEIVER_CALL_COUNTER_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };

    /// Stack of currently executing component instance keys for the current thread.
    ///
    /// Unlike `current_instance_key()`, this remains stable for the whole
    /// component body even when nested control-flow groups are entered.
    static CURRENT_COMPONENT_INSTANCE_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };

    /// One-shot instance-logic-id override used by subtree replay.
    ///
    /// When set, the next `push_current_node` call will consume this value as
    /// the component instance logic id instead of deriving it from parent stacks.
    static NEXT_NODE_INSTANCE_LOGIC_ID_OVERRIDE: RefCell<Option<u64>> = const { RefCell::new(None) };

    /// Active reactive-dirty instance-key set for the current build pass.
    static BUILD_DIRTY_INSTANCE_KEYS_STACK: RefCell<Vec<Arc<HashSet<u64>>>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn compute_context_slot_key() -> (u64, u64) {
    let instance_logic_id = current_instance_logic_id();
    let group_path_hash = current_group_path_hash();

    let call_counter = CONTEXT_CALL_COUNTER_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        debug_assert!(
            !stack.is_empty(),
            "CONTEXT_CALL_COUNTER_STACK is empty; provide_context must be called inside a component"
        );
        let counter = *stack
            .last()
            .expect("CONTEXT_CALL_COUNTER_STACK should not be empty");
        *stack
            .last_mut()
            .expect("CONTEXT_CALL_COUNTER_STACK should not be empty") += 1;
        counter
    });

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
    value: Option<Arc<dyn Any + Send + Sync>>,
    last_alive_epoch: u64,
    retained: bool,
}

#[derive(Default)]
struct InstanceSlotCursor {
    previous_order: Vec<SlotHandle>,
    current_order: Vec<SlotHandle>,
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

    fn reset(&mut self) {
        self.entries.clear();
        self.key_to_slot.clear();
        self.cursors_by_instance_logic_id.clear();
        self.epoch = 0;
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

static SLOT_TABLE: OnceLock<RwLock<SlotTable>> = OnceLock::new();

fn slot_table() -> &'static RwLock<SlotTable> {
    SLOT_TABLE.get_or_init(|| RwLock::new(SlotTable::default()))
}

#[derive(Default)]
struct LayoutDirtyTracker {
    previous_layout_specs_by_node: HashMap<u64, Box<dyn LayoutSpecDyn>>,
    frame_layout_specs_by_node: HashMap<u64, Box<dyn LayoutSpecDyn>>,
    pending_self_dirty_nodes: HashSet<u64>,
    ready_self_dirty_nodes: HashSet<u64>,
    previous_children_by_node: HashMap<u64, Vec<u64>>,
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
    pub fn_name: String,
    pub replay: ComponentReplayData,
}

#[derive(Default)]
struct ComponentReplayTracker {
    previous_nodes: HashMap<u64, ReplayNodeSnapshot>,
    current_nodes: HashMap<u64, ReplayNodeSnapshot>,
}

static COMPONENT_REPLAY_TRACKER: OnceLock<RwLock<ComponentReplayTracker>> = OnceLock::new();

fn component_replay_tracker() -> &'static RwLock<ComponentReplayTracker> {
    COMPONENT_REPLAY_TRACKER.get_or_init(|| RwLock::new(ComponentReplayTracker::default()))
}

pub(crate) fn begin_frame_component_replay_tracking() {
    component_replay_tracker().write().current_nodes.clear();
}

pub(crate) fn finalize_frame_component_replay_tracking() {
    let mut tracker = component_replay_tracker().write();
    tracker.previous_nodes = std::mem::take(&mut tracker.current_nodes);
}

pub(crate) fn finalize_frame_component_replay_tracking_partial() {
    let mut tracker = component_replay_tracker().write();
    let current = std::mem::take(&mut tracker.current_nodes);
    tracker.previous_nodes.extend(current);
}

pub(crate) fn reset_component_replay_tracking() {
    *component_replay_tracker().write() = ComponentReplayTracker::default();
}

pub(crate) fn previous_component_replay_nodes() -> HashMap<u64, ReplayNodeSnapshot> {
    component_replay_tracker().read().previous_nodes.clone()
}

pub(crate) fn remove_previous_component_replay_nodes(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    let mut tracker = component_replay_tracker().write();
    tracker
        .previous_nodes
        .retain(|instance_key, _| !instance_keys.contains(instance_key));
    tracker
        .current_nodes
        .retain(|instance_key, _| !instance_keys.contains(instance_key));
}

#[derive(Default)]
struct BuildInvalidationTracker {
    dirty_instance_keys: HashSet<u64>,
}

#[derive(Default)]
pub(crate) struct BuildInvalidationSet {
    pub dirty_instance_keys: HashSet<u64>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct StateReadDependencyKey {
    // Keep generation in the dependency key to avoid ABA when a slot is recycled
    // and later reused for another state value.
    slot: SlotHandle,
    generation: u64,
}

#[derive(Default)]
struct StateReadDependencyTracker {
    readers_by_state: HashMap<StateReadDependencyKey, HashSet<u64>>,
    states_by_reader: HashMap<u64, HashSet<StateReadDependencyKey>>,
}

static BUILD_INVALIDATION_TRACKER: OnceLock<RwLock<BuildInvalidationTracker>> = OnceLock::new();
static STATE_READ_DEPENDENCY_TRACKER: OnceLock<RwLock<StateReadDependencyTracker>> =
    OnceLock::new();
type RedrawWaker = Arc<dyn Fn() + Send + Sync + 'static>;
static REDRAW_WAKER: OnceLock<RwLock<Option<RedrawWaker>>> = OnceLock::new();
static REDRAW_REQUEST_PENDING: AtomicBool = AtomicBool::new(false);

fn build_invalidation_tracker() -> &'static RwLock<BuildInvalidationTracker> {
    BUILD_INVALIDATION_TRACKER.get_or_init(|| RwLock::new(BuildInvalidationTracker::default()))
}

fn state_read_dependency_tracker() -> &'static RwLock<StateReadDependencyTracker> {
    STATE_READ_DEPENDENCY_TRACKER.get_or_init(|| RwLock::new(StateReadDependencyTracker::default()))
}

fn redraw_waker() -> &'static RwLock<Option<RedrawWaker>> {
    REDRAW_WAKER.get_or_init(|| RwLock::new(None))
}

fn schedule_runtime_redraw() {
    if REDRAW_REQUEST_PENDING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    let callback = redraw_waker().read().clone();
    if let Some(callback) = callback {
        callback();
    } else {
        REDRAW_REQUEST_PENDING.store(false, Ordering::Release);
    }
}

pub(crate) fn install_redraw_waker(callback: RedrawWaker) {
    *redraw_waker().write() = Some(callback);
    if REDRAW_REQUEST_PENDING.load(Ordering::Acquire) {
        let callback = redraw_waker().read().clone();
        if let Some(callback) = callback {
            callback();
        }
    }
}

pub(crate) fn clear_redraw_waker() {
    *redraw_waker().write() = None;
    REDRAW_REQUEST_PENDING.store(false, Ordering::Release);
}

pub(crate) fn consume_scheduled_redraw() {
    REDRAW_REQUEST_PENDING.store(false, Ordering::Release);
}

fn current_component_instance_key_from_scope() -> Option<u64> {
    CURRENT_COMPONENT_INSTANCE_STACK.with(|stack| stack.borrow().last().copied())
}

fn take_next_node_instance_logic_id_override() -> Option<u64> {
    NEXT_NODE_INSTANCE_LOGIC_ID_OVERRIDE.with(|slot| slot.borrow_mut().take())
}

/// Runs `f` inside a replay scope restored from a previously recorded component
/// snapshot.
///
/// The replay scope restores:
/// - the control-flow group path active at the original call site
/// - a one-shot instance-logic-id override for the replayed component root
pub(crate) fn with_replay_scope<R>(
    instance_logic_id: u64,
    group_path: &[u64],
    f: impl FnOnce() -> R,
) -> R {
    struct ReplayScopeGuard {
        previous_group_path: Option<Vec<u64>>,
        previous_instance_logic_id_override: Option<Option<u64>>,
    }

    impl Drop for ReplayScopeGuard {
        fn drop(&mut self) {
            if let Some(previous_group_path) = self.previous_group_path.take() {
                GROUP_PATH_STACK.with(|stack| {
                    *stack.borrow_mut() = previous_group_path;
                });
            }
            if let Some(previous_instance_logic_id_override) =
                self.previous_instance_logic_id_override.take()
            {
                NEXT_NODE_INSTANCE_LOGIC_ID_OVERRIDE.with(|slot| {
                    *slot.borrow_mut() = previous_instance_logic_id_override;
                });
            }
        }
    }

    let previous_group_path = GROUP_PATH_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        std::mem::replace(&mut *stack, group_path.to_vec())
    });
    let previous_instance_logic_id_override = NEXT_NODE_INSTANCE_LOGIC_ID_OVERRIDE.with(|slot| {
        let mut slot = slot.borrow_mut();
        (*slot).replace(instance_logic_id)
    });
    let _guard = ReplayScopeGuard {
        previous_group_path: Some(previous_group_path),
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
            BUILD_DIRTY_INSTANCE_KEYS_STACK.with(|stack| {
                let mut stack = stack.borrow_mut();
                let popped = stack.pop();
                debug_assert!(
                    popped.is_some(),
                    "BUILD_DIRTY_INSTANCE_KEYS_STACK underflow: attempted to pop from empty stack"
                );
            });
            self.popped = true;
        }
    }

    BUILD_DIRTY_INSTANCE_KEYS_STACK.with(|stack| {
        stack
            .borrow_mut()
            .push(Arc::new(dirty_instance_keys.clone()));
    });
    let _guard = BuildDirtyScopeGuard { popped: false };
    f()
}

pub(crate) fn is_instance_key_build_dirty(instance_key: u64) -> bool {
    BUILD_DIRTY_INSTANCE_KEYS_STACK.with(|stack| {
        stack
            .borrow()
            .last()
            .is_some_and(|dirty_instance_keys| dirty_instance_keys.contains(&instance_key))
    })
}

pub(crate) fn current_component_instance_key_in_scope() -> Option<u64> {
    current_component_instance_key_from_scope()
}

pub(crate) fn record_component_invalidation_for_instance_key(instance_key: u64) {
    let inserted = build_invalidation_tracker()
        .write()
        .dirty_instance_keys
        .insert(instance_key);
    if inserted {
        schedule_runtime_redraw();
    }
}

fn track_state_read_dependency(slot: SlotHandle, generation: u64) {
    if !matches!(current_phase(), Some(RuntimePhase::Build)) {
        return;
    }
    let Some(reader_instance_key) = current_component_instance_key_from_scope() else {
        return;
    };

    let key = StateReadDependencyKey { slot, generation };
    let mut tracker = state_read_dependency_tracker().write();
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

fn state_read_subscribers(slot: SlotHandle, generation: u64) -> Vec<u64> {
    let key = StateReadDependencyKey { slot, generation };
    state_read_dependency_tracker()
        .read()
        .readers_by_state
        .get(&key)
        .map(|readers| readers.iter().copied().collect())
        .unwrap_or_default()
}

pub(crate) fn remove_state_read_dependencies(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    let mut tracker = state_read_dependency_tracker().write();
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

pub(crate) fn reset_state_read_dependencies() {
    *state_read_dependency_tracker().write() = StateReadDependencyTracker::default();
}

pub(crate) fn take_build_invalidations() -> BuildInvalidationSet {
    let mut tracker = build_invalidation_tracker().write();
    BuildInvalidationSet {
        dirty_instance_keys: std::mem::take(&mut tracker.dirty_instance_keys),
    }
}

pub(crate) fn reset_build_invalidations() {
    *build_invalidation_tracker().write() = BuildInvalidationTracker::default();
}

pub(crate) fn remove_build_invalidations(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    build_invalidation_tracker()
        .write()
        .dirty_instance_keys
        .retain(|instance_key| !instance_keys.contains(instance_key));
}

pub(crate) fn has_pending_build_invalidations() -> bool {
    !build_invalidation_tracker()
        .read()
        .dirty_instance_keys
        .is_empty()
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

static FRAME_CLOCK_TRACKER: OnceLock<Mutex<FrameClockTracker>> = OnceLock::new();

fn frame_clock_tracker() -> &'static Mutex<FrameClockTracker> {
    FRAME_CLOCK_TRACKER.get_or_init(|| Mutex::new(FrameClockTracker::default()))
}

pub(crate) fn begin_frame_clock(now: Instant) {
    let mut tracker = frame_clock_tracker().lock();
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

pub(crate) fn reset_frame_clock() {
    *frame_clock_tracker().lock() = FrameClockTracker::default();
}

pub(crate) fn has_pending_frame_nanos_receivers() -> bool {
    !frame_clock_tracker().lock().receivers.is_empty()
}

pub(crate) fn tick_frame_nanos_receivers() {
    let frame_nanos = frame_clock_tracker().lock().current_frame_nanos;
    let mut tracker = frame_clock_tracker().lock();
    tracker.receivers.retain(|_, receiver| {
        matches!(
            (receiver.callback)(frame_nanos),
            FrameNanosControl::Continue
        )
    });
}

pub(crate) fn remove_frame_nanos_receivers(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    frame_clock_tracker()
        .lock()
        .receivers
        .retain(|_, receiver| !instance_keys.contains(&receiver.owner_instance_key));
}

pub(crate) fn clear_frame_nanos_receivers() {
    frame_clock_tracker().lock().receivers.clear();
}

/// Returns the timestamp of the current frame, if available.
///
/// The value is set by the renderer at frame begin.
pub fn current_frame_time() -> Option<Instant> {
    frame_clock_tracker().lock().current_frame_time
}

/// Returns the current frame timestamp in nanoseconds from runtime origin.
pub fn current_frame_nanos() -> u64 {
    frame_clock_tracker().lock().current_frame_nanos
}

/// Returns the elapsed time since the previous frame.
pub fn frame_delta() -> Duration {
    frame_clock_tracker().lock().frame_delta
}

fn ensure_frame_receive_phase() {
    match current_phase() {
        Some(RuntimePhase::Build) | Some(RuntimePhase::Input) => {}
        Some(RuntimePhase::Measure) => {
            panic!("receive_frame_nanos must not be called inside measure")
        }
        None => panic!(
            "receive_frame_nanos must be called inside a tessera component build or input handler"
        ),
    }
}

fn current_component_instance_key_for_receiver() -> Option<u64> {
    current_component_instance_key_from_scope()
}

fn compute_frame_nanos_receiver_key() -> FrameNanosReceiverKey {
    let instance_logic_id = current_instance_logic_id();
    let group_path_hash = current_group_path_hash();

    let call_counter = FRAME_RECEIVER_CALL_COUNTER_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        debug_assert!(
            !stack.is_empty(),
            "FRAME_RECEIVER_CALL_COUNTER_STACK is empty; receive_frame_nanos must be called inside a component"
        );
        let counter = *stack
            .last()
            .expect("FRAME_RECEIVER_CALL_COUNTER_STACK should not be empty");
        *stack
            .last_mut()
            .expect("FRAME_RECEIVER_CALL_COUNTER_STACK should not be empty") += 1;
        counter
    });

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

    let owner_instance_key = current_component_instance_key_for_receiver()
        .unwrap_or_else(|| panic!("receive_frame_nanos requires an active component node context"));
    let key = compute_frame_nanos_receiver_key();

    let mut tracker = frame_clock_tracker().lock();
    tracker
        .receivers
        .entry(key)
        .or_insert_with(|| FrameNanosReceiver {
            owner_instance_key,
            callback: Box::new(callback),
        });
}

pub(crate) fn drop_slots_for_instance_logic_ids(instance_logic_ids: &HashSet<u64>) {
    if instance_logic_ids.is_empty() {
        return;
    }

    let mut table = slot_table().write();
    let mut freed: Vec<(SlotHandle, SlotKey)> = Vec::new();
    for (slot, entry) in table.entries.iter() {
        if !instance_logic_ids.contains(&entry.key.instance_logic_id) {
            continue;
        }
        // `retain` state must survive subtree removal and route switches.
        if entry.retained {
            continue;
        }
        freed.push((slot, entry.key));
    }
    for (slot, key) in freed {
        table.entries.remove(slot);
        table.key_to_slot.remove(&key);
    }
    for instance_logic_id in instance_logic_ids {
        table.cursors_by_instance_logic_id.remove(instance_logic_id);
    }
}

static LAYOUT_DIRTY_TRACKER: OnceLock<RwLock<LayoutDirtyTracker>> = OnceLock::new();

fn layout_dirty_tracker() -> &'static RwLock<LayoutDirtyTracker> {
    LAYOUT_DIRTY_TRACKER.get_or_init(|| RwLock::new(LayoutDirtyTracker::default()))
}

fn record_layout_spec_dirty(instance_key: u64, layout_spec: &dyn LayoutSpecDyn) {
    if current_phase() != Some(RuntimePhase::Build) {
        return;
    }
    let mut tracker = layout_dirty_tracker().write();
    let changed = match tracker.previous_layout_specs_by_node.get(&instance_key) {
        Some(previous) => !previous.dyn_eq(layout_spec),
        None => true,
    };
    if changed {
        tracker.pending_self_dirty_nodes.insert(instance_key);
    }
    tracker
        .frame_layout_specs_by_node
        .insert(instance_key, layout_spec.clone_box());
}

pub(crate) fn begin_frame_layout_dirty_tracking() {
    let mut tracker = layout_dirty_tracker().write();
    tracker.frame_layout_specs_by_node.clear();
    tracker.pending_self_dirty_nodes.clear();
}

pub(crate) fn finalize_frame_layout_dirty_tracking() {
    let mut tracker = layout_dirty_tracker().write();
    tracker.ready_self_dirty_nodes = std::mem::take(&mut tracker.pending_self_dirty_nodes);
    tracker.previous_layout_specs_by_node = std::mem::take(&mut tracker.frame_layout_specs_by_node);
}

pub(crate) fn take_layout_self_dirty_nodes() -> HashSet<u64> {
    std::mem::take(&mut layout_dirty_tracker().write().ready_self_dirty_nodes)
}

pub(crate) fn reset_layout_dirty_tracking() {
    *layout_dirty_tracker().write() = LayoutDirtyTracker::default();
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
        fn_name: node.fn_name.clone(),
        replay,
    };
    component_replay_tracker()
        .write()
        .current_nodes
        .insert(snapshot.instance_key, snapshot);
}

pub(crate) fn reconcile_layout_structure(
    current_children_by_node: &HashMap<u64, Vec<u64>>,
) -> StructureReconcileResult {
    let mut tracker = layout_dirty_tracker().write();
    let previous_children_by_node = &tracker.previous_children_by_node;

    let mut changed_nodes = HashSet::default();
    let mut removed_nodes = HashSet::default();

    for (node, current_children) in current_children_by_node {
        match previous_children_by_node.get(node) {
            Some(previous_children) if previous_children == current_children => {}
            _ => {
                changed_nodes.insert(*node);
            }
        }
    }

    for node in previous_children_by_node.keys().copied() {
        if !current_children_by_node.contains_key(&node) {
            changed_nodes.insert(node);
            removed_nodes.insert(node);
        }
    }

    tracker.previous_children_by_node = current_children_by_node.clone();
    StructureReconcileResult {
        changed_nodes,
        removed_nodes,
    }
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
    fn load_entry(&self) -> Arc<dyn Any + Send + Sync> {
        let table = slot_table().read();
        let entry = table
            .entries
            .get(self.slot)
            .unwrap_or_else(|| panic!("State points to freed slot: {:?}", self.slot));

        if entry.generation != self.generation {
            panic!(
                "State is stale (slot {:?}, generation {}, current generation {})",
                self.slot, self.generation, entry.generation
            );
        }

        if entry.key.type_id != TypeId::of::<T>() {
            panic!(
                "State type mismatch for slot {:?}: expected {}, stored {:?}",
                self.slot,
                std::any::type_name::<T>(),
                entry.key.type_id
            );
        }

        entry
            .value
            .as_ref()
            .unwrap_or_else(|| panic!("State slot {:?} has been cleared", self.slot))
            .clone()
    }

    fn load_lock(&self) -> Arc<RwLock<T>> {
        self.load_entry()
            .downcast::<RwLock<T>>()
            .unwrap_or_else(|_| panic!("State slot {:?} downcast failed", self.slot))
    }

    /// Execute a closure with a shared reference to the stored value.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        track_state_read_dependency(self.slot, self.generation);
        let lock = self.load_lock();
        let guard = lock.read();
        f(&guard)
    }

    /// Execute a closure with a mutable reference to the stored value.
    #[track_caller]
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let lock = self.load_lock();

        let result = {
            let mut guard = lock.write();
            f(&mut guard)
        };

        let subscribers = state_read_subscribers(self.slot, self.generation);
        for instance_key in subscribers {
            record_component_invalidation_for_instance_key(instance_key);
        }
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

/// Global singleton instance of the [`TesseraRuntime`].
static TESSERA_RUNTIME: OnceLock<RwLock<TesseraRuntime>> = OnceLock::new();

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
        f(&TESSERA_RUNTIME
            .get_or_init(|| RwLock::new(Self::default()))
            .read())
    }

    /// Executes a closure with an exclusive, mutable reference to the runtime.
    pub fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        f(&mut TESSERA_RUNTIME
            .get_or_init(|| RwLock::new(Self::default()))
            .write())
    }

    /// Get the current window size in physical pixels.
    pub fn window_size(&self) -> [u32; 2] {
        self.window_size
    }

    /// Sets identity fields for the current component node.
    #[doc(hidden)]
    pub fn set_current_node_identity(&mut self, instance_key: u64, instance_logic_id: u64) {
        if let Some(node) = self.component_tree.current_node_mut() {
            node.instance_key = instance_key;
            node.instance_logic_id = instance_logic_id;
        } else {
            debug_assert!(
                false,
                "set_current_node_identity must be called inside a component build"
            );
        }
    }

    /// Stores replay metadata for the current component node.
    #[doc(hidden)]
    pub fn set_current_component_replay<P>(
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
            let tracker = component_replay_tracker().read();
            let previous = tracker.previous_nodes.get(&instance_key)?;
            if previous.instance_logic_id != instance_logic_id {
                return None;
            }
            if previous.replay.props.equals(props) {
                Some(previous.replay.clone())
            } else {
                None
            }
        });

        if let Some((instance_key, instance_logic_id)) = current_node_info
            && let Some(replay) = previous_replay.clone()
            && !is_instance_key_build_dirty(instance_key)
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

    /// Sets the layout spec for the current component node.
    #[doc(hidden)]
    pub fn set_current_layout_spec<S>(&mut self, spec: S)
    where
        S: LayoutSpec,
    {
        if let Some(node) = self.component_tree.current_node_mut() {
            node.layout_spec = Box::new(spec) as Box<dyn LayoutSpecDyn>;
        } else {
            debug_assert!(
                false,
                "set_current_layout_spec must be called inside a component build"
            );
        }
    }

    /// Records the final layout spec snapshot for the current node.
    #[doc(hidden)]
    pub fn finalize_current_layout_spec_dirty(&mut self) {
        if let Some(node) = self.component_tree.current_node() {
            record_layout_spec_dirty(node.instance_key, node.layout_spec.as_ref());
        } else {
            debug_assert!(
                false,
                "finalize_current_layout_spec_dirty must be called inside a component build"
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

pub struct CurrentComponentInstanceGuard {
    popped: bool,
}

/// Execution phase for `remember` usage checks.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RuntimePhase {
    /// Component render/build phase (allowed for `remember`).
    Build,
    /// Measurement phase (disallowed for `remember`).
    Measure,
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

/// Push the given node id as the current executing component for this thread.
pub fn push_current_node(
    node_id: NodeId,
    component_type_id: u64,
    fn_name: &str,
) -> NodeContextGuard {
    #[cfg(not(feature = "profiling"))]
    let _ = fn_name;
    #[allow(unused_variables)]
    let parent_node_id = NODE_CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let parent = stack.last().copied();
        stack.push(node_id);
        parent
    });

    // Get the parent's call index and increment it
    // This distinguishes multiple calls to the same component (e.g., foo(1);
    // foo(2);)
    let (parent_call_index, parent_instance_logic_id) = INSTANCE_CALL_COUNTER_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let index = stack.last().copied().unwrap_or(0);
        if let Some(last) = stack.last_mut() {
            *last += 1;
        }
        let parent_id = INSTANCE_LOGIC_ID_STACK.with(|s| s.borrow().last().copied().unwrap_or(0));
        (index, parent_id)
    });

    // Combine component_type_id with parent_instance_logic_id and
    // parent_call_index to create a stable instance logic id. This ensures:
    // 1. foo(1) and foo(2) get different logic_ids (via parent_call_index)
    // 2. Components in different container instances get different logic_ids (via
    //    parent_instance_logic_id)
    let instance_salt = if let Some(key_hash) = current_instance_key_override() {
        hash_components(&[&key_hash, &parent_call_index])
    } else {
        parent_call_index
    };

    let instance_logic_id =
        if let Some(instance_logic_id_override) = take_next_node_instance_logic_id_override() {
            instance_logic_id_override
        } else if parent_call_index == 0
            && parent_instance_logic_id == 0
            && current_instance_key_override().is_none()
        {
            component_type_id
        } else {
            hash_components(&[
                &component_type_id,
                &parent_instance_logic_id,
                &instance_salt,
            ])
        };

    INSTANCE_LOGIC_ID_STACK.with(|stack| stack.borrow_mut().push(instance_logic_id));

    // Push a new call counter layer for this component's internal remember calls
    CALL_COUNTER_STACK.with(|stack| stack.borrow_mut().push(0));

    // Push a new call counter layer for this component's internal context providers
    CONTEXT_CALL_COUNTER_STACK.with(|stack| stack.borrow_mut().push(0));

    // Push a new call counter layer for this component's child instance identity
    INSTANCE_CALL_COUNTER_STACK.with(|stack| stack.borrow_mut().push(0));

    // Push a new call counter layer for frame-nanos receivers.
    FRAME_RECEIVER_CALL_COUNTER_STACK.with(|stack| stack.borrow_mut().push(0));

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
    let parent_node_id = NODE_CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let parent = stack.last().copied();
        stack.push(node_id);
        parent
    });

    // Keep child call counters balanced with push/pop semantics even when we
    // restore an explicit instance logic id.
    INSTANCE_CALL_COUNTER_STACK.with(|stack| {
        if let Some(last) = stack.borrow_mut().last_mut() {
            *last += 1;
        }
    });

    INSTANCE_LOGIC_ID_STACK.with(|stack| stack.borrow_mut().push(instance_logic_id));
    CALL_COUNTER_STACK.with(|stack| stack.borrow_mut().push(0));
    CONTEXT_CALL_COUNTER_STACK.with(|stack| stack.borrow_mut().push(0));
    INSTANCE_CALL_COUNTER_STACK.with(|stack| stack.borrow_mut().push(0));
    FRAME_RECEIVER_CALL_COUNTER_STACK.with(|stack| stack.borrow_mut().push(0));

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
    CURRENT_COMPONENT_INSTANCE_STACK.with(|stack| stack.borrow_mut().push(instance_key));
    CurrentComponentInstanceGuard { popped: false }
}

fn pop_current_component_instance_key() {
    CURRENT_COMPONENT_INSTANCE_STACK.with(|stack| {
        let popped = stack.borrow_mut().pop();
        debug_assert!(
            popped.is_some(),
            "Attempted to pop current component instance key from an empty stack"
        );
    });
}

fn pop_current_node() {
    NODE_CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let popped = stack.pop();
        debug_assert!(
            popped.is_some(),
            "Attempted to pop current node from an empty stack"
        );
    });
    // Pop this component's call counter layer
    CALL_COUNTER_STACK.with(|stack| {
        let popped = stack.borrow_mut().pop();
        debug_assert!(
            popped.is_some(),
            "CALL_COUNTER_STACK underflow: attempted to pop from empty stack"
        );
    });

    // Pop this component's context call counter layer
    CONTEXT_CALL_COUNTER_STACK.with(|stack| {
        let popped = stack.borrow_mut().pop();
        debug_assert!(
            popped.is_some(),
            "CONTEXT_CALL_COUNTER_STACK underflow: attempted to pop from empty stack"
        );
    });

    INSTANCE_CALL_COUNTER_STACK.with(|stack| {
        let popped = stack.borrow_mut().pop();
        debug_assert!(
            popped.is_some(),
            "INSTANCE_CALL_COUNTER_STACK underflow: attempted to pop from empty stack"
        );
    });

    FRAME_RECEIVER_CALL_COUNTER_STACK.with(|stack| {
        let popped = stack.borrow_mut().pop();
        debug_assert!(
            popped.is_some(),
            "FRAME_RECEIVER_CALL_COUNTER_STACK underflow: attempted to pop from empty stack"
        );
    });
}

/// Get the node id at the top of the thread-local component stack.
pub fn current_node_id() -> Option<NodeId> {
    NODE_CONTEXT_STACK.with(|stack| stack.borrow().last().copied())
}

fn current_instance_logic_id_opt() -> Option<u64> {
    INSTANCE_LOGIC_ID_STACK.with(|stack| stack.borrow().last().copied())
}

/// Returns the current component instance logic id.
#[doc(hidden)]
pub fn current_instance_logic_id() -> u64 {
    current_instance_logic_id_opt()
        .expect("current_instance_logic_id must be called inside a component")
}

/// Returns the instance key for the current component call site.
#[doc(hidden)]
pub fn current_instance_key() -> u64 {
    let instance_logic_id = current_instance_logic_id_opt()
        .expect("current_instance_key must be called inside a component");
    let group_path_hash = current_group_path_hash();
    hash_components(&[&instance_logic_id, &group_path_hash])
}

fn pop_instance_logic_id() {
    INSTANCE_LOGIC_ID_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let _ = stack.pop();
    });
}

/// Push an execution phase for the current thread.
pub fn push_phase(phase: RuntimePhase) -> PhaseGuard {
    PHASE_STACK.with(|stack| stack.borrow_mut().push(phase));
    PhaseGuard { popped: false }
}

fn pop_phase() {
    PHASE_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let popped = stack.pop();
        debug_assert!(
            popped.is_some(),
            "Attempted to pop execution phase from an empty stack"
        );
    });
}

pub(crate) fn current_phase() -> Option<RuntimePhase> {
    PHASE_STACK.with(|stack| stack.borrow().last().copied())
}

/// Push a group id onto the thread-local control-flow stack.
pub(crate) fn push_group_id(group_id: u64) {
    GROUP_PATH_STACK.with(|stack| stack.borrow_mut().push(group_id));
}

/// Pop a group id from the thread-local control-flow stack.
pub(crate) fn pop_group_id(expected_group_id: u64) {
    GROUP_PATH_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if let Some(popped) = stack.pop() {
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
    GROUP_PATH_STACK.with(|stack| stack.borrow().clone())
}

fn current_group_path_hash() -> u64 {
    let group_path = current_group_path();
    hash_components(&[&group_path])
}

fn current_instance_key_override() -> Option<u64> {
    INSTANCE_KEY_STACK.with(|stack| stack.borrow().last().copied())
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
        Self { group_id }
    }
}

impl Drop for GroupGuard {
    fn drop(&mut self) {
        pop_group_id(self.group_id);
    }
}

/// RAII guard that sets a stable instance key for the duration of a block.
pub struct InstanceKeyGuard {
    key_hash: u64,
}

impl InstanceKeyGuard {
    /// Push a key hash for instance identity and reset the instance call
    /// counter.
    pub fn new(key_hash: u64) -> Self {
        INSTANCE_KEY_STACK.with(|stack| stack.borrow_mut().push(key_hash));
        INSTANCE_CALL_COUNTER_STACK.with(|stack| stack.borrow_mut().push(0));
        Self { key_hash }
    }
}

impl Drop for InstanceKeyGuard {
    fn drop(&mut self) {
        INSTANCE_CALL_COUNTER_STACK.with(|stack| {
            let popped = stack.borrow_mut().pop();
            debug_assert!(
                popped.is_some(),
                "INSTANCE_CALL_COUNTER_STACK underflow: attempted to pop from empty stack"
            );
        });
        INSTANCE_KEY_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            let popped = stack.pop();
            debug_assert_eq!(
                popped,
                Some(self.key_hash),
                "Unbalanced InstanceKeyGuard stack"
            );
        });
    }
}

fn hash_components<H: Hash>(parts: &[&H]) -> u64 {
    let mut hasher = rustc_hash::FxHasher::default();
    for part in parts {
        part.hash(&mut hasher);
    }
    hasher.finish()
}

fn compute_slot_key<K: Hash>(key: &K) -> (u64, u64) {
    let instance_logic_id = current_instance_logic_id();
    let group_path = current_group_path();
    let group_path_hash = hash_components(&[&group_path]);
    let key_hash = hash_components(&[key]);

    // Get the call counter to distinguish multiple remember calls within the same
    // component Note: instance_logic_id already distinguishes different component
    // instances (foo(1) vs foo(2)) and group_path_hash handles nested control
    // flow (if/loop)
    let call_counter = CALL_COUNTER_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        debug_assert!(
            !stack.is_empty(),
            "CALL_COUNTER_STACK is empty; remember must be called inside a component"
        );
        let counter = *stack
            .last()
            .expect("CALL_COUNTER_STACK should not be empty");
        *stack
            .last_mut()
            .expect("CALL_COUNTER_STACK should not be empty") += 1;
        counter
    });

    let slot_hash = hash_components(&[&group_path_hash, &key_hash, &call_counter]);
    (instance_logic_id, slot_hash)
}

pub(crate) fn ensure_build_phase() {
    match current_phase() {
        Some(RuntimePhase::Build) => {}
        Some(RuntimePhase::Measure) => {
            panic!("remember must not be called inside measure; move state to component render")
        }
        Some(RuntimePhase::Input) => {
            panic!(
                "remember must not be called inside input_handler; move state to component render"
            )
        }
        None => panic!(
            "remember must be called inside a tessera component. Ensure you're calling this from within a function annotated with #[tessera]."
        ),
    }
}

/// Start a new state-slot epoch for the current recomposition pass.
pub fn begin_recompose_slot_epoch() {
    slot_table().write().begin_epoch();
}

/// Reset all slot buffers (used on suspension).
pub fn reset_slots() {
    slot_table().write().reset();
}

pub(crate) fn recycle_recomposed_slots_for_instance_logic_ids(instance_logic_ids: &HashSet<u64>) {
    if instance_logic_ids.is_empty() {
        return;
    }

    let mut table = slot_table().write();
    let epoch = table.epoch;
    let mut freed: Vec<(SlotHandle, SlotKey)> = Vec::new();

    for (slot, entry) in table.entries.iter() {
        if !instance_logic_ids.contains(&entry.key.instance_logic_id) {
            continue;
        }
        // Skip if touched in this recomposition pass or marked as retained.
        if entry.last_alive_epoch == epoch || entry.retained {
            continue;
        }
        freed.push((slot, entry.key));
    }

    for (slot, key) in freed {
        table.entries.remove(slot);
        table.key_to_slot.remove(&key);
    }
}

pub(crate) fn live_slot_instance_logic_ids() -> HashSet<u64> {
    let table = slot_table().read();
    table
        .entries
        .iter()
        .map(|(_, entry)| entry.key.instance_logic_id)
        .collect()
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
/// This function returns a `State<T>` handle that internally uses an
/// `Arc<RwLock<T>>`. Use `with`, `with_mut`, `get`, or `set` to read or update
/// the value without handling synchronization primitives directly.
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
    ensure_build_phase();
    let (instance_logic_id, slot_hash) = compute_slot_key(&key);
    let type_id = TypeId::of::<T>();
    let slot_key = SlotKey {
        instance_logic_id,
        slot_hash,
        type_id,
    };

    let mut table = slot_table().write();
    let mut init_opt = Some(init);
    if let Some(slot) = table.try_fast_slot_lookup(slot_key) {
        let epoch = table.epoch;
        let generation = {
            let entry = table
                .entries
                .get_mut(slot)
                .expect("slot entry should exist");

            if entry.key.type_id != slot_key.type_id {
                panic!(
                    "remember_with_key type mismatch: expected {}, found {:?}",
                    std::any::type_name::<T>(),
                    entry.key.type_id
                );
            }

            entry.last_alive_epoch = epoch;
            if entry.value.is_none() {
                let init_fn = init_opt
                    .take()
                    .expect("remember_with_key init called more than once");
                entry.value = Some(Arc::new(RwLock::new(init_fn())));
                entry.generation = entry.generation.wrapping_add(1);
            }
            entry.generation
        };

        State::new(slot, generation)
    } else if let Some(slot) = table.key_to_slot.get(&slot_key).copied() {
        table.record_slot_usage_slow(instance_logic_id, slot);
        let epoch = table.epoch;
        let generation = {
            let entry = table
                .entries
                .get_mut(slot)
                .expect("slot entry should exist");

            if entry.key.type_id != slot_key.type_id {
                panic!(
                    "remember_with_key type mismatch: expected {}, found {:?}",
                    std::any::type_name::<T>(),
                    entry.key.type_id
                );
            }

            entry.last_alive_epoch = epoch;
            if entry.value.is_none() {
                let init_fn = init_opt
                    .take()
                    .expect("remember_with_key init called more than once");
                entry.value = Some(Arc::new(RwLock::new(init_fn())));
                entry.generation = entry.generation.wrapping_add(1);
            }
            entry.generation
        };

        State::new(slot, generation)
    } else {
        let epoch = table.epoch;
        let init_fn = init_opt
            .take()
            .expect("remember_with_key init called more than once");
        let generation = 1u64;
        let slot = table.entries.insert(SlotEntry {
            key: slot_key,
            generation,
            value: Some(Arc::new(RwLock::new(init_fn()))),
            last_alive_epoch: epoch,
            retained: false,
        });

        table.key_to_slot.insert(slot_key, slot);
        table.record_slot_usage_slow(instance_logic_id, slot);
        State::new(slot, generation)
    }
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
/// This function returns a `State<T>` handle that internally uses an
/// `Arc<RwLock<T>>`. Use `with`, `with_mut`, `get`, or `set` to read or update
/// the value without handling synchronization primitives directly.
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
/// This function returns a `State<T>` handle that internally uses an
/// `Arc<RwLock<T>>`. Use `with`, `with_mut`, `get`, or `set` to read or update
/// the value without handling synchronization primitives directly.
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
    ensure_build_phase();
    let (instance_logic_id, slot_hash) = compute_slot_key(&key);
    let type_id = TypeId::of::<T>();
    let slot_key = SlotKey {
        instance_logic_id,
        slot_hash,
        type_id,
    };

    let mut table = slot_table().write();
    let mut init_opt = Some(init);
    if let Some(slot) = table.try_fast_slot_lookup(slot_key) {
        let epoch = table.epoch;
        let generation = {
            let entry = table
                .entries
                .get_mut(slot)
                .expect("slot entry should exist");

            if entry.key.type_id != slot_key.type_id {
                panic!(
                    "retain_with_key type mismatch: expected {}, found {:?}",
                    std::any::type_name::<T>(),
                    entry.key.type_id
                );
            }

            entry.last_alive_epoch = epoch;
            entry.retained = true;
            if entry.value.is_none() {
                let init_fn = init_opt
                    .take()
                    .expect("retain_with_key init called more than once");
                entry.value = Some(Arc::new(RwLock::new(init_fn())));
                entry.generation = entry.generation.wrapping_add(1);
            }

            entry.generation
        };

        State::new(slot, generation)
    } else if let Some(slot) = table.key_to_slot.get(&slot_key).copied() {
        table.record_slot_usage_slow(instance_logic_id, slot);
        let epoch = table.epoch;
        let generation = {
            let entry = table
                .entries
                .get_mut(slot)
                .expect("slot entry should exist");

            if entry.key.type_id != slot_key.type_id {
                panic!(
                    "retain_with_key type mismatch: expected {}, found {:?}",
                    std::any::type_name::<T>(),
                    entry.key.type_id
                );
            }

            entry.last_alive_epoch = epoch;
            entry.retained = true;
            if entry.value.is_none() {
                let init_fn = init_opt
                    .take()
                    .expect("retain_with_key init called more than once");
                entry.value = Some(Arc::new(RwLock::new(init_fn())));
                entry.generation = entry.generation.wrapping_add(1);
            }

            entry.generation
        };

        State::new(slot, generation)
    } else {
        let epoch = table.epoch;
        let init_fn = init_opt
            .take()
            .expect("retain_with_key init called more than once");
        let generation = 1u64;
        let slot = table.entries.insert(SlotEntry {
            key: slot_key,
            generation,
            value: Some(Arc::new(RwLock::new(init_fn()))),
            last_alive_epoch: epoch,
            retained: true,
        });

        table.key_to_slot.insert(slot_key, slot);
        table.record_slot_usage_slow(instance_logic_id, slot);
        State::new(slot, generation)
    }
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
/// This function returns a `State<T>` handle that internally uses an
/// `Arc<RwLock<T>>`. Use `with`, `with_mut`, `get`, or `set` to read or update
/// the value without handling synchronization primitives directly.
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
/// #[derive(Clone, PartialEq)]
/// struct MyListArgs {
///     items: Vec<String>,
/// }
///
/// #[tessera]
/// fn my_list(args: &MyListArgs) {
///     for item in args.items.iter() {
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
    use super::*;

    #[test]
    fn frame_receiver_uses_component_scope_instance_key() {
        let _instance_guard = push_current_component_instance_key(7);
        assert_eq!(current_component_instance_key_for_receiver(), Some(7));
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

        frame_clock_tracker().lock().receivers.insert(
            FrameNanosReceiverKey {
                instance_logic_id: 1,
                receiver_hash: 1,
            },
            FrameNanosReceiver {
                owner_instance_key: 123,
                callback: Box::new(|_| FrameNanosControl::Stop),
            },
        );

        tick_frame_nanos_receivers();
        assert!(frame_clock_tracker().lock().receivers.is_empty());
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

    #[test]
    fn with_replay_scope_restores_group_path_and_override() {
        GROUP_PATH_STACK.with(|stack| {
            *stack.borrow_mut() = vec![1, 2, 3];
        });
        NEXT_NODE_INSTANCE_LOGIC_ID_OVERRIDE.with(|slot| {
            *slot.borrow_mut() = Some(9);
        });

        with_replay_scope(42, &[7, 8], || {
            assert_eq!(current_group_path(), vec![7, 8]);
            assert_eq!(take_next_node_instance_logic_id_override(), Some(42));
            assert_eq!(take_next_node_instance_logic_id_override(), None);
        });

        assert_eq!(current_group_path(), vec![1, 2, 3]);
        let restored_override = NEXT_NODE_INSTANCE_LOGIC_ID_OVERRIDE.with(|slot| *slot.borrow());
        assert_eq!(restored_override, Some(9));
    }

    #[test]
    fn with_replay_scope_restores_on_panic() {
        GROUP_PATH_STACK.with(|stack| {
            *stack.borrow_mut() = vec![5];
        });
        NEXT_NODE_INSTANCE_LOGIC_ID_OVERRIDE.with(|slot| {
            *slot.borrow_mut() = None;
        });

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_replay_scope(77, &[10], || {
                assert_eq!(current_group_path(), vec![10]);
                panic!("expected panic");
            });
        }));
        assert!(result.is_err());

        assert_eq!(current_group_path(), vec![5]);
        let restored_override = NEXT_NODE_INSTANCE_LOGIC_ID_OVERRIDE.with(|slot| *slot.borrow());
        assert_eq!(restored_override, None);
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
            value: Some(Arc::new(RwLock::new(10_i32))),
            last_alive_epoch: 0,
            retained: true,
        });
        let drop_slot = table.entries.insert(SlotEntry {
            key: drop_key,
            generation: 1,
            value: Some(Arc::new(RwLock::new(20_i32))),
            last_alive_epoch: 0,
            retained: false,
        });
        table.key_to_slot.insert(keep_key, keep_slot);
        table.key_to_slot.insert(drop_key, drop_slot);
        *slot_table().write() = table;

        let mut stale = HashSet::default();
        stale.insert(7_u64);
        drop_slots_for_instance_logic_ids(&stale);

        let table = slot_table().read();
        assert!(table.entries.get(keep_slot).is_some());
        assert!(table.key_to_slot.get(&keep_key).is_some());
        assert!(table.entries.get(drop_slot).is_none());
        assert!(table.key_to_slot.get(&drop_key).is_none());
    }
}
