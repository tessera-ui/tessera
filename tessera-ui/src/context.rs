//! # Context provider
//!
//! ## Usage
//!
//! Provide themes and configuration objects to descendant components.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    sync::{Arc, OnceLock},
};

use im::HashMap as ImHashMap;
use parking_lot::RwLock;

use crate::runtime::{
    RuntimePhase, compute_context_slot_key, current_component_instance_key_in_scope, current_phase,
    ensure_build_phase, record_component_invalidation_for_instance_key,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ContextSnapshotEntry {
    slot: u32,
    generation: u64,
    key: SlotKey,
}

pub(crate) type ContextMap = ImHashMap<TypeId, ContextSnapshotEntry>;

thread_local! {
    static CONTEXT_STACK: RefCell<Vec<ContextMap>> = RefCell::new(vec![ContextMap::new()]);
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
struct SlotKey {
    logic_id: u64,
    slot_hash: u64,
    type_id: TypeId,
}

struct SlotEntry {
    key: SlotKey,
    generation: u64,
    value: Option<Arc<dyn Any + Send + Sync>>,
    last_alive_epoch: u64,
}

#[derive(Default)]
struct SlotTable {
    entries: Vec<SlotEntry>,
    free_list: Vec<u32>,
    key_to_slot: HashMap<SlotKey, u32>,
    epoch: u64,
}

impl SlotTable {
    fn begin_frame(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
    }
}

static SLOT_TABLE: OnceLock<RwLock<SlotTable>> = OnceLock::new();

fn slot_table() -> &'static RwLock<SlotTable> {
    SLOT_TABLE.get_or_init(|| RwLock::new(SlotTable::default()))
}

#[derive(Default)]
struct ContextSnapshotTracker {
    previous_by_instance_key: HashMap<u64, ContextMap>,
    current_by_instance_key: HashMap<u64, ContextMap>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ContextReadDependencyKey {
    slot: u32,
    generation: u64,
}

#[derive(Default)]
struct ContextReadDependencyTracker {
    readers_by_context: HashMap<ContextReadDependencyKey, HashSet<u64>>,
    contexts_by_reader: HashMap<u64, HashSet<ContextReadDependencyKey>>,
}

static CONTEXT_SNAPSHOT_TRACKER: OnceLock<RwLock<ContextSnapshotTracker>> = OnceLock::new();
static CONTEXT_READ_DEPENDENCY_TRACKER: OnceLock<RwLock<ContextReadDependencyTracker>> =
    OnceLock::new();

fn context_snapshot_tracker() -> &'static RwLock<ContextSnapshotTracker> {
    CONTEXT_SNAPSHOT_TRACKER.get_or_init(|| RwLock::new(ContextSnapshotTracker::default()))
}

fn context_read_dependency_tracker() -> &'static RwLock<ContextReadDependencyTracker> {
    CONTEXT_READ_DEPENDENCY_TRACKER
        .get_or_init(|| RwLock::new(ContextReadDependencyTracker::default()))
}

fn current_context_map() -> ContextMap {
    CONTEXT_STACK.with(|stack| {
        stack
            .borrow()
            .last()
            .cloned()
            .unwrap_or_else(ContextMap::new)
    })
}

fn resolve_snapshot_entry(entry: ContextSnapshotEntry) -> Option<ContextSnapshotEntry> {
    let table = slot_table().read();
    let live_entry = table.entries.get(entry.slot as usize);
    if let Some(live_entry) = live_entry
        && live_entry.value.is_some()
        && live_entry.key == entry.key
    {
        return Some(ContextSnapshotEntry {
            slot: entry.slot,
            generation: live_entry.generation,
            key: entry.key,
        });
    }

    let slot = table.key_to_slot.get(&entry.key).copied()?;
    let live_entry = table.entries.get(slot as usize)?;
    live_entry.value.as_ref()?;
    Some(ContextSnapshotEntry {
        slot,
        generation: live_entry.generation,
        key: entry.key,
    })
}

fn normalize_context_snapshot(snapshot: &ContextMap) -> ContextMap {
    snapshot
        .iter()
        .fold(ContextMap::new(), |acc, (type_id, entry)| {
            if let Some(resolved) = resolve_snapshot_entry(*entry) {
                acc.update(*type_id, resolved)
            } else {
                acc
            }
        })
}

pub(crate) fn begin_frame_component_context_tracking() {
    context_snapshot_tracker()
        .write()
        .current_by_instance_key
        .clear();
}

pub(crate) fn finalize_frame_component_context_tracking() {
    let mut tracker = context_snapshot_tracker().write();
    tracker.previous_by_instance_key = std::mem::take(&mut tracker.current_by_instance_key);
}

pub(crate) fn finalize_frame_component_context_tracking_partial() {
    let mut tracker = context_snapshot_tracker().write();
    let current = std::mem::take(&mut tracker.current_by_instance_key);
    tracker.previous_by_instance_key.extend(current);
}

pub(crate) fn reset_component_context_tracking() {
    *context_snapshot_tracker().write() = ContextSnapshotTracker::default();
}

pub(crate) fn previous_component_context_snapshots() -> HashMap<u64, ContextMap> {
    context_snapshot_tracker()
        .read()
        .previous_by_instance_key
        .clone()
}

pub(crate) fn context_from_previous_snapshot_for_instance<T>(
    instance_key: u64,
) -> Option<Context<T>>
where
    T: Send + Sync + 'static,
{
    let tracker = context_snapshot_tracker().read();
    let map = tracker.previous_by_instance_key.get(&instance_key)?;
    map.get(&TypeId::of::<T>())
        .copied()
        .and_then(resolve_snapshot_entry)
        .map(|entry| Context::new(entry.slot, entry.generation))
}

pub(crate) fn remove_previous_component_context_snapshots(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    let mut tracker = context_snapshot_tracker().write();
    tracker
        .previous_by_instance_key
        .retain(|instance_key, _| !instance_keys.contains(instance_key));
    tracker
        .current_by_instance_key
        .retain(|instance_key, _| !instance_keys.contains(instance_key));
}

pub(crate) fn remove_context_read_dependencies(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    let mut tracker = context_read_dependency_tracker().write();
    for instance_key in instance_keys {
        let Some(context_keys) = tracker.contexts_by_reader.remove(instance_key) else {
            continue;
        };
        for context_key in context_keys {
            let mut remove_entry = false;
            if let Some(readers) = tracker.readers_by_context.get_mut(&context_key) {
                readers.remove(instance_key);
                remove_entry = readers.is_empty();
            }
            if remove_entry {
                tracker.readers_by_context.remove(&context_key);
            }
        }
    }
}

pub(crate) fn reset_context_read_dependencies() {
    *context_read_dependency_tracker().write() = ContextReadDependencyTracker::default();
}

#[doc(hidden)]
pub fn record_current_context_snapshot_for(instance_key: u64) {
    context_snapshot_tracker()
        .write()
        .current_by_instance_key
        .insert(instance_key, current_context_map());
}

pub(crate) fn with_context_snapshot<R>(snapshot: &ContextMap, f: impl FnOnce() -> R) -> R {
    struct ContextSnapshotGuard {
        previous_stack: Option<Vec<ContextMap>>,
    }

    impl Drop for ContextSnapshotGuard {
        fn drop(&mut self) {
            if let Some(previous_stack) = self.previous_stack.take() {
                CONTEXT_STACK.with(|stack| {
                    *stack.borrow_mut() = previous_stack;
                });
            }
        }
    }

    let normalized_snapshot = normalize_context_snapshot(snapshot);
    let previous_stack = CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        std::mem::replace(&mut *stack, vec![normalized_snapshot])
    });
    let _guard = ContextSnapshotGuard {
        previous_stack: Some(previous_stack),
    };

    f()
}

fn track_context_read_dependency(slot: u32, generation: u64) {
    if !matches!(current_phase(), Some(RuntimePhase::Build)) {
        return;
    }
    let Some(reader_instance_key) = current_component_instance_key_in_scope() else {
        return;
    };

    let key = ContextReadDependencyKey { slot, generation };
    let mut tracker = context_read_dependency_tracker().write();
    tracker
        .readers_by_context
        .entry(key)
        .or_default()
        .insert(reader_instance_key);
    tracker
        .contexts_by_reader
        .entry(reader_instance_key)
        .or_default()
        .insert(key);
}

fn context_read_subscribers(slot: u32, generation: u64) -> Vec<u64> {
    let key = ContextReadDependencyKey { slot, generation };
    context_read_dependency_tracker()
        .read()
        .readers_by_context
        .get(&key)
        .map(|readers| readers.iter().copied().collect())
        .unwrap_or_default()
}

pub(crate) fn begin_frame_context_slots() {
    // Start a new context-slot epoch for the current recomposition pass.
    slot_table().write().begin_frame();
    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.clear();
        stack.push(ContextMap::new());
    });
}

pub(crate) fn recycle_recomposed_context_slots_for_logic_ids(logic_ids: &HashSet<u64>) {
    if logic_ids.is_empty() {
        return;
    }

    // Recycle untouched context slots for logic ids recomposed in this pass.
    let mut table = slot_table().write();
    let epoch = table.epoch;
    let mut freed: Vec<(u32, SlotKey)> = Vec::new();
    for (slot, entry) in table.entries.iter_mut().enumerate() {
        if !logic_ids.contains(&entry.key.logic_id) {
            continue;
        }
        if entry.value.is_none() {
            continue;
        }
        if entry.last_alive_epoch == epoch {
            continue;
        }

        freed.push((slot as u32, entry.key));
        entry.value = None;
        entry.generation = entry.generation.wrapping_add(1);
        entry.last_alive_epoch = 0;
    }

    for (slot, key) in freed {
        table.key_to_slot.remove(&key);
        table.free_list.push(slot);
    }
}

pub(crate) fn live_context_slot_logic_ids() -> HashSet<u64> {
    let table = slot_table().read();
    table
        .entries
        .iter()
        .filter(|entry| entry.value.is_some())
        .map(|entry| entry.key.logic_id)
        .collect()
}

pub(crate) fn drop_context_slots_for_logic_ids(logic_ids: &HashSet<u64>) {
    if logic_ids.is_empty() {
        return;
    }

    let mut table = slot_table().write();
    let mut freed: Vec<(u32, SlotKey)> = Vec::new();
    for (slot, entry) in table.entries.iter_mut().enumerate() {
        if entry.value.is_none() {
            continue;
        }
        if !logic_ids.contains(&entry.key.logic_id) {
            continue;
        }
        freed.push((slot as u32, entry.key));
        entry.value = None;
        entry.generation = entry.generation.wrapping_add(1);
        entry.last_alive_epoch = 0;
    }

    for (slot, key) in freed {
        table.key_to_slot.remove(&key);
        table.free_list.push(slot);
    }
}

/// Handle to a context value created by [`provide_context`] and retrieved by
/// [`use_context`].
///
/// Handles are validated with a slot generation token so stale references fail
/// fast if their slot has been recycled.
///
/// # Examples
///
/// ```
/// use tessera_ui::{provide_context, tessera, use_context};
///
/// #[derive(Default)]
/// struct Theme {
///     color: String,
/// }
///
/// #[tessera]
/// fn root() {
///     provide_context(
///         || Theme {
///             color: "blue".into(),
///         },
///         || {
///             child();
///         },
///     );
/// }
///
/// #[tessera]
/// fn child() {
///     let theme = use_context::<Theme>().expect("Theme must be provided");
///     theme.with(|t| println!("Color: {}", t.color));
/// }
/// ```
pub struct Context<T> {
    slot: u32,
    generation: u64,
    _marker: PhantomData<T>,
}

impl<T> Copy for Context<T> {}

impl<T> Clone for Context<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Context<T> {
    fn new(slot: u32, generation: u64) -> Self {
        Self {
            slot,
            generation,
            _marker: PhantomData,
        }
    }
}

impl<T> Context<T>
where
    T: Send + Sync + 'static,
{
    fn load_entry(&self) -> Arc<dyn Any + Send + Sync> {
        let table = slot_table().read();
        let entry = table
            .entries
            .get(self.slot as usize)
            .unwrap_or_else(|| panic!("Context points to freed slot: {}", self.slot));

        if entry.generation != self.generation {
            panic!(
                "Context is stale (slot {}, generation {}, current generation {})",
                self.slot, self.generation, entry.generation
            );
        }

        if entry.key.type_id != TypeId::of::<T>() {
            panic!(
                "Context type mismatch for slot {}: expected {}, stored {:?}",
                self.slot,
                std::any::type_name::<T>(),
                entry.key.type_id
            );
        }

        entry
            .value
            .as_ref()
            .unwrap_or_else(|| panic!("Context slot {} has been recycled", self.slot))
            .clone()
    }

    fn load_lock(&self) -> Arc<RwLock<T>> {
        self.load_entry()
            .downcast::<RwLock<T>>()
            .unwrap_or_else(|_| panic!("Context slot {} downcast failed", self.slot))
    }

    /// Execute a closure with a shared reference to the context value.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        track_context_read_dependency(self.slot, self.generation);
        let lock = self.load_lock();
        let guard = lock.read();
        f(&guard)
    }

    /// Execute a closure with a mutable reference to the context value.
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let lock = self.load_lock();
        let result = {
            let mut guard = lock.write();
            f(&mut guard)
        };
        let subscribers = context_read_subscribers(self.slot, self.generation);
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

    /// Set the context value.
    pub fn set(&self, value: T) {
        self.with_mut(|v| *v = value);
    }

    /// Replace the context value and return the old value.
    pub fn replace(&self, value: T) -> T {
        self.with_mut(|v| std::mem::replace(v, value))
    }
}

fn push_context_layer(type_id: TypeId, slot: u32, generation: u64, key: SlotKey) {
    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let parent = stack.last().cloned().unwrap_or_else(ContextMap::new);
        let next = parent.update(
            type_id,
            ContextSnapshotEntry {
                slot,
                generation,
                key,
            },
        );
        stack.push(next);
    });
}

fn pop_context_layer() {
    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let popped = stack.pop();
        debug_assert!(popped.is_some(), "Context stack underflow");
        if stack.is_empty() {
            stack.push(ContextMap::new());
        }
    });
}

/// Provides a typed context value for the duration of the given closure.
///
/// The `init` closure is evaluated only when the context slot is created (or
/// re-created after being recycled).
///
/// This is intended for use inside component build functions only.
///
/// # Examples
///
/// ```
/// use tessera_ui::{Color, provide_context, tessera, use_context};
///
/// #[derive(Default)]
/// struct Theme {
///     primary: Color,
/// }
///
/// #[tessera]
/// fn root() {
///     provide_context(
///         || Theme {
///             primary: Color::RED,
///         },
///         || {
///             leaf();
///         },
///     );
/// }
///
/// #[tessera]
/// fn leaf() {
///     let theme = use_context::<Theme>().expect("Theme must be provided");
///     theme.with(|t| assert_eq!(t.primary, Color::RED));
/// }
/// ```
pub fn provide_context<T, I, F, R>(init: I, f: F) -> R
where
    T: Send + Sync + 'static,
    I: FnOnce() -> T,
    F: FnOnce() -> R,
{
    ensure_build_phase();

    let (logic_id, slot_hash) = compute_context_slot_key();
    let type_id = TypeId::of::<T>();
    let slot_key = SlotKey {
        logic_id,
        slot_hash,
        type_id,
    };

    let (slot, generation) = {
        let mut table = slot_table().write();
        let epoch = table.epoch;

        if let Some(slot) = table.key_to_slot.get(&slot_key).copied() {
            let entry = table
                .entries
                .get_mut(slot as usize)
                .expect("context slot entry should exist");
            entry.last_alive_epoch = epoch;

            if entry.value.is_none() {
                entry.value = Some(Arc::new(RwLock::new(init())));
                entry.generation = entry.generation.wrapping_add(1);
            }

            let generation = entry.generation;
            (slot, generation)
        } else if let Some(slot) = table.free_list.pop() {
            let entry = table
                .entries
                .get_mut(slot as usize)
                .expect("context slot entry should exist");

            entry.key = slot_key;
            entry.value = Some(Arc::new(RwLock::new(init())));
            entry.last_alive_epoch = epoch;

            let generation = entry.generation;
            table.key_to_slot.insert(slot_key, slot);
            (slot, generation)
        } else {
            let generation = 0;
            let slot = table.entries.len() as u32;
            table.entries.push(SlotEntry {
                key: slot_key,
                generation,
                value: Some(Arc::new(RwLock::new(init()))),
                last_alive_epoch: epoch,
            });
            table.key_to_slot.insert(slot_key, slot);
            (slot, generation)
        }
    };

    push_context_layer(type_id, slot, generation, slot_key);

    struct ContextScopeGuard;
    impl Drop for ContextScopeGuard {
        fn drop(&mut self) {
            pop_context_layer();
        }
    }

    let guard = ContextScopeGuard;
    let result = f();
    drop(guard);
    result
}

/// Reads a typed context value from the current scope.
///
/// # Examples
///
/// ```
/// use tessera_ui::{Color, tessera, use_context};
///
/// #[derive(Default)]
/// struct Theme {
///     primary: Color,
/// }
///
/// #[tessera]
/// fn component() {
///     let theme = use_context::<Theme>();
///     assert!(theme.is_none());
/// }
/// ```
pub fn use_context<T>() -> Option<Context<T>>
where
    T: Send + Sync + 'static,
{
    ensure_build_phase();

    CONTEXT_STACK.with(|stack| {
        let stack = stack.borrow();
        let map = stack
            .last()
            .expect("Context stack must always contain at least one layer");
        map.get(&TypeId::of::<T>())
            .copied()
            .map(|entry| Context::new(entry.slot, entry.generation))
    })
}
// (legacy comment removed)

#[cfg(test)]
mod tests {
    use std::{
        any::TypeId,
        sync::{Arc, Mutex, OnceLock},
    };

    use parking_lot::RwLock;

    use super::{
        CONTEXT_STACK, ContextMap, ContextSnapshotEntry, SlotEntry, SlotKey, SlotTable, slot_table,
        with_context_snapshot,
    };
    use crate::runtime::{RuntimePhase, push_phase};

    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("test lock poisoned")
    }

    fn reset_test_state() {
        *slot_table().write() = SlotTable::default();
        CONTEXT_STACK.with(|stack| {
            *stack.borrow_mut() = vec![ContextMap::new()];
        });
    }

    #[test]
    fn with_context_snapshot_restores_stack_after_panic() {
        let _lock = test_lock();
        reset_test_state();

        let base_layer = ContextMap::new();
        let parent_layer = base_layer.update(
            TypeId::of::<u8>(),
            ContextSnapshotEntry {
                slot: 1,
                generation: 2,
                key: SlotKey {
                    logic_id: 7,
                    slot_hash: 11,
                    type_id: TypeId::of::<u8>(),
                },
            },
        );
        let original_stack = vec![base_layer, parent_layer];
        CONTEXT_STACK.with(|stack| {
            *stack.borrow_mut() = original_stack.clone();
        });

        let snapshot = ContextMap::new().update(
            TypeId::of::<u32>(),
            ContextSnapshotEntry {
                slot: 3,
                generation: 4,
                key: SlotKey {
                    logic_id: 17,
                    slot_hash: 19,
                    type_id: TypeId::of::<u32>(),
                },
            },
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_context_snapshot(&snapshot, || {
                CONTEXT_STACK.with(|stack| {
                    let stack = stack.borrow();
                    assert_eq!(stack.len(), 1);
                    assert!(stack.last().is_some());
                });
                panic!("expected panic in context snapshot test");
            });
        }));
        assert!(result.is_err());

        CONTEXT_STACK.with(|stack| {
            assert_eq!(*stack.borrow(), original_stack);
        });
    }

    #[test]
    fn with_context_snapshot_remaps_stale_generation() {
        let _lock = test_lock();
        reset_test_state();

        let key = SlotKey {
            logic_id: 31,
            slot_hash: 37,
            type_id: TypeId::of::<u8>(),
        };
        {
            let mut table = slot_table().write();
            *table = SlotTable::default();
            table.entries.push(SlotEntry {
                key,
                generation: 2,
                value: Some(Arc::new(RwLock::new(42_u8))),
                last_alive_epoch: 1,
            });
            table.key_to_slot.insert(key, 0);
        }

        let snapshot = ContextMap::new().update(
            TypeId::of::<u8>(),
            ContextSnapshotEntry {
                slot: 0,
                generation: 1,
                key,
            },
        );

        let _phase_guard = push_phase(RuntimePhase::Build);
        with_context_snapshot(&snapshot, || {
            let context = super::use_context::<u8>().expect("context should be remapped");
            assert_eq!(context.get(), 42);
        });
    }

    #[test]
    fn with_context_snapshot_remaps_stale_slot_by_key() {
        let _lock = test_lock();
        reset_test_state();

        let key = SlotKey {
            logic_id: 41,
            slot_hash: 43,
            type_id: TypeId::of::<u16>(),
        };
        let old_key = SlotKey {
            logic_id: 47,
            slot_hash: 53,
            type_id: TypeId::of::<u16>(),
        };
        {
            let mut table = slot_table().write();
            *table = SlotTable::default();
            table.entries.push(SlotEntry {
                key: old_key,
                generation: 9,
                value: None,
                last_alive_epoch: 0,
            });
            table.entries.push(SlotEntry {
                key,
                generation: 4,
                value: Some(Arc::new(RwLock::new(7_u16))),
                last_alive_epoch: 2,
            });
            table.key_to_slot.insert(key, 1);
        }

        let snapshot = ContextMap::new().update(
            TypeId::of::<u16>(),
            ContextSnapshotEntry {
                slot: 0,
                generation: 3,
                key,
            },
        );

        let _phase_guard = push_phase(RuntimePhase::Build);
        with_context_snapshot(&snapshot, || {
            let context = super::use_context::<u16>().expect("context should be remapped by key");
            assert_eq!(context.get(), 7);
        });
    }
}
