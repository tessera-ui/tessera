//! # Context provider
//!
//! ## Usage
//!
//! Provide themes and configuration objects to descendant components.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
};

use im::HashMap as ImHashMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    execution_context::{with_execution_context, with_execution_context_mut},
    read_dependencies::ReadDependencies,
    runtime::{
        RuntimePhase, compute_context_slot_key, current_phase,
        current_replay_boundary_instance_key_from_scope, ensure_build_phase,
        record_replay_boundary_invalidation_for_instance_key,
    },
    session::with_runtime_globals,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ContextSnapshotEntry {
    slot: u32,
    generation: u64,
    key: SlotKey,
}

type ContextBindings = ImHashMap<TypeId, ContextSnapshotEntry>;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ContextMap(Option<Rc<ContextBindings>>);

impl ContextMap {
    fn new() -> Self {
        Self(None)
    }

    fn from_bindings(bindings: ContextBindings) -> Self {
        Self((!bindings.is_empty()).then(|| Rc::new(bindings)))
    }

    fn get(&self, type_id: &TypeId) -> Option<&ContextSnapshotEntry> {
        self.0.as_deref()?.get(type_id)
    }

    fn update(&self, type_id: TypeId, entry: ContextSnapshotEntry) -> Self {
        let mut bindings = self.0.as_deref().cloned().unwrap_or_default();
        bindings.insert(type_id, entry);
        Self::from_bindings(bindings)
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
struct SlotKey {
    instance_logic_id: u64,
    slot_hash: u64,
    type_id: TypeId,
}

struct SlotEntry {
    key: SlotKey,
    generation: u64,
    value: Option<Rc<dyn Any>>,
    last_alive_epoch: u64,
}

#[derive(Default)]
struct SlotTable {
    entries: Vec<SlotEntry>,
    free_list: Vec<u32>,
    key_to_slot: HashMap<SlotKey, u32>,
    slots_by_owner: HashMap<u64, HashSet<u32>>,
    epoch: u64,
}

impl SlotTable {
    fn index_slot(&mut self, key: SlotKey, slot: u32) {
        self.key_to_slot.insert(key, slot);
        self.slots_by_owner
            .entry(key.instance_logic_id)
            .or_default()
            .insert(slot);
    }

    fn unindex_slot(&mut self, key: SlotKey, slot: u32) {
        self.key_to_slot.remove(&key);
        if let Some(slots) = self.slots_by_owner.get_mut(&key.instance_logic_id) {
            slots.remove(&slot);
            if slots.is_empty() {
                self.slots_by_owner.remove(&key.instance_logic_id);
            }
        }
    }

    fn begin_epoch(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
    }
}

fn with_slot_table<R>(f: impl FnOnce(&SlotTable) -> R) -> R {
    with_runtime_globals(|globals| f(&globals.context.slot_table.borrow()))
}

fn with_slot_table_mut<R>(f: impl FnOnce(&mut SlotTable) -> R) -> R {
    with_runtime_globals(|globals| f(&mut globals.context.slot_table.borrow_mut()))
}

pub(crate) fn reset_context_slots() {
    let removed = with_slot_table_mut(|table| {
        let mut removed = Vec::new();
        for entry in &mut table.entries {
            if let Some(value) = entry.value.take() {
                removed.push(value);
            }
            entry.generation = entry.generation.wrapping_add(1);
        }
        table.key_to_slot.clear();
        table.slots_by_owner.clear();
        table.free_list = (0..table.entries.len() as u32).collect();
        removed
    });
    drop(removed);
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ContextReadDependencyKey {
    slot: u32,
    generation: u64,
}

type ContextReadDependencyTracker = ReadDependencies<ContextReadDependencyKey, u64>;

fn with_context_read_dependency_tracker<R>(
    f: impl FnOnce(&ContextReadDependencyTracker) -> R,
) -> R {
    with_runtime_globals(|globals| f(&globals.context.read_dependency_tracker.borrow()))
}

fn with_context_read_dependency_tracker_mut<R>(
    f: impl FnOnce(&mut ContextReadDependencyTracker) -> R,
) -> R {
    with_runtime_globals(|globals| f(&mut globals.context.read_dependency_tracker.borrow_mut()))
}

#[derive(Default)]
pub(crate) struct ContextGlobals {
    slot_table: RefCell<SlotTable>,
    read_dependency_tracker: RefCell<ContextReadDependencyTracker>,
}

pub(crate) fn current_context_map() -> ContextMap {
    with_execution_context(|context| context.context_map.clone())
}

fn resolve_snapshot_entry(
    table: &SlotTable,
    entry: ContextSnapshotEntry,
) -> Option<ContextSnapshotEntry> {
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

pub(crate) fn normalize_context_snapshot(snapshot: &ContextMap) -> ContextMap {
    let Some(bindings) = snapshot.0.as_deref() else {
        return ContextMap::new();
    };
    let normalized = with_slot_table(|table| {
        let mut updated = None;
        for (&type_id, &entry) in bindings {
            let resolved = resolve_snapshot_entry(table, entry);
            if resolved == Some(entry) {
                continue;
            }
            let next = updated.get_or_insert_with(|| bindings.clone());
            if let Some(resolved) = resolved {
                next.insert(type_id, resolved);
            } else {
                next.remove(&type_id);
            }
        }
        updated
    });
    normalized.map_or_else(|| snapshot.clone(), ContextMap::from_bindings)
}

pub(crate) fn context_from_previous_snapshot_for_instance<T>(
    instance_key: u64,
) -> Option<Context<T>>
where
    T: 'static,
{
    let map = crate::runtime::previous_component_context(instance_key)?;
    map.get(&TypeId::of::<T>())
        .copied()
        .and_then(|entry| with_slot_table(|table| resolve_snapshot_entry(table, entry)))
        .map(|entry| Context::new(entry.slot, entry.generation))
}

pub(crate) fn remove_context_read_dependencies(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    with_context_read_dependency_tracker_mut(|tracker| tracker.remove_readers(instance_keys));
}

pub(crate) fn reset_context_read_dependencies() {
    with_context_read_dependency_tracker_mut(|tracker| {
        *tracker = ContextReadDependencyTracker::default();
    });
}

pub(crate) fn with_context_map<R>(map: ContextMap, f: impl FnOnce() -> R) -> R {
    struct RestoreContext(ContextMap);

    impl Drop for RestoreContext {
        fn drop(&mut self) {
            with_execution_context_mut(|context| {
                std::mem::swap(&mut context.context_map, &mut self.0);
            });
        }
    }

    let previous =
        with_execution_context_mut(|context| std::mem::replace(&mut context.context_map, map));
    let _restore = RestoreContext(previous);
    f()
}

fn track_context_read_dependency(slot: u32, generation: u64) {
    if !matches!(current_phase(), Some(RuntimePhase::Build)) {
        return;
    }
    let Some(reader_instance_key) = current_replay_boundary_instance_key_from_scope() else {
        return;
    };

    let key = ContextReadDependencyKey { slot, generation };
    with_context_read_dependency_tracker_mut(|tracker| tracker.record(key, reader_instance_key));
}

fn context_read_subscribers(slot: u32, generation: u64) -> Vec<u64> {
    let key = ContextReadDependencyKey { slot, generation };
    with_context_read_dependency_tracker(|tracker| {
        tracker
            .readers(&key)
            .map(Iterator::collect)
            .unwrap_or_default()
    })
}

pub(crate) fn begin_recompose_context_slot_epoch() {
    // Start a new context-slot epoch for the current recomposition pass.
    with_slot_table_mut(SlotTable::begin_epoch);
    with_execution_context_mut(|context| context.context_map = ContextMap::new());
}

pub(crate) fn recycle_recomposed_context_slots_for_instance_logic_ids(
    instance_logic_ids: &HashSet<u64>,
) {
    if instance_logic_ids.is_empty() {
        return;
    }

    // Recycle untouched context slots for logic ids recomposed in this pass.
    let removed = with_slot_table_mut(|table| {
        let epoch = table.epoch;
        let mut freed: Vec<(u32, SlotKey)> = Vec::new();
        let mut removed = Vec::new();
        let slots: Vec<_> = instance_logic_ids
            .iter()
            .flat_map(|id| table.slots_by_owner.get(id).into_iter().flatten())
            .copied()
            .collect();
        for slot in slots {
            let entry = &mut table.entries[slot as usize];
            if !instance_logic_ids.contains(&entry.key.instance_logic_id) {
                continue;
            }
            if entry.value.is_none() {
                continue;
            }
            if entry.last_alive_epoch == epoch {
                continue;
            }

            freed.push((slot, entry.key));
            removed.push(entry.value.take());
            entry.generation = entry.generation.wrapping_add(1);
            entry.last_alive_epoch = 0;
        }

        for (slot, key) in freed {
            table.unindex_slot(key, slot);
            table.free_list.push(slot);
        }
        removed
    });
    // User destructors may read context after all slot metadata is consistent.
    drop(removed);
}

pub(crate) fn live_context_slot_instance_logic_ids() -> HashSet<u64> {
    with_slot_table(|table| table.slots_by_owner.keys().copied().collect())
}

pub(crate) fn drop_context_slots_for_instance_logic_ids(instance_logic_ids: &HashSet<u64>) {
    if instance_logic_ids.is_empty() {
        return;
    }

    let removed = with_slot_table_mut(|table| {
        let mut freed: Vec<(u32, SlotKey)> = Vec::new();
        let mut removed = Vec::new();
        let slots: Vec<_> = instance_logic_ids
            .iter()
            .flat_map(|id| table.slots_by_owner.get(id).into_iter().flatten())
            .copied()
            .collect();
        for slot in slots {
            let entry = &mut table.entries[slot as usize];
            if entry.value.is_none() {
                continue;
            }
            if !instance_logic_ids.contains(&entry.key.instance_logic_id) {
                continue;
            }
            freed.push((slot, entry.key));
            removed.push(entry.value.take());
            entry.generation = entry.generation.wrapping_add(1);
            entry.last_alive_epoch = 0;
        }

        for (slot, key) in freed {
            table.unindex_slot(key, slot);
            table.free_list.push(slot);
        }
        removed
    });
    drop(removed);
}

/// Handle to inherited visual defaults created by [`provide_context`].
///
/// Accessing a stale handle fails rather than reading a replacement value.
pub struct Context<T> {
    session: u128,
    slot: u32,
    generation: u64,
    _marker: PhantomData<*const T>,
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
            session: crate::session::current_session_identity().expect("active context session"),
            slot,
            generation,
            _marker: PhantomData,
        }
    }
}

impl<T> Context<T>
where
    T: 'static,
{
    fn load_entry(&self) -> Rc<dyn Any> {
        assert_eq!(
            crate::session::current_session_identity(),
            Some(self.session),
            "context belongs to a different runtime session"
        );
        with_slot_table(|table| {
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
        })
    }

    fn load_cell(&self) -> Rc<RefCell<T>> {
        self.load_entry()
            .downcast::<RefCell<T>>()
            .unwrap_or_else(|_| panic!("Context slot {} downcast failed", self.slot))
    }

    /// Execute a closure with a shared reference to the context value.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        assert_eq!(
            crate::session::current_session_identity(),
            Some(self.session),
            "context belongs to a different runtime session"
        );
        track_context_read_dependency(self.slot, self.generation);
        let cell = self.load_cell();
        let value = cell.borrow();
        f(&value)
    }

    /// Get a cloned value. Requires `T: Clone`.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with(Clone::clone)
    }
}

/// Provides `value` to descendants built by `f`.
///
/// Each build compares `value` with the previously provided one; a change
/// invalidates the descendants that read it.
///
/// This is intended for use inside component build functions only.
///
/// # Examples
///
/// ```
/// use tessera_ui::{Color, provide_context, tessera, use_context};
///
/// #[derive(Clone, PartialEq)]
/// struct Theme {
///     primary: Color,
/// }
///
/// #[tessera]
/// fn root() {
///     provide_context(
///         Theme {
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
pub fn provide_context<T, R>(value: T, f: impl FnOnce() -> R) -> R
where
    T: PartialEq + 'static,
{
    ensure_build_phase();

    let (instance_logic_id, slot_hash) = compute_context_slot_key();
    let type_id = TypeId::of::<T>();
    let slot_key = SlotKey {
        instance_logic_id,
        slot_hash,
        type_id,
    };

    fn cell<T: 'static>(value: T) -> Rc<dyn Any> {
        Rc::new(RefCell::new(value))
    }

    let (slot, generation, changed) = with_slot_table_mut(|table| {
        let epoch = table.epoch;

        if let Some(slot) = table.key_to_slot.get(&slot_key).copied() {
            let entry = table
                .entries
                .get_mut(slot as usize)
                .expect("context slot entry should exist");
            entry.last_alive_epoch = epoch;
            let changed = match entry
                .value
                .as_ref()
                .and_then(|stored| stored.downcast_ref::<RefCell<T>>())
            {
                Some(stored) => {
                    let changed = *stored.borrow() != value;
                    if changed {
                        *stored.borrow_mut() = value;
                    }
                    changed
                }
                None => {
                    entry.value = Some(cell(value));
                    entry.generation = entry.generation.wrapping_add(1);
                    false
                }
            };
            (slot, entry.generation, changed)
        } else if let Some(slot) = table.free_list.pop() {
            let entry = table
                .entries
                .get_mut(slot as usize)
                .expect("context slot entry should exist");
            entry.key = slot_key;
            entry.value = Some(cell(value));
            entry.last_alive_epoch = epoch;
            let generation = entry.generation;
            table.index_slot(slot_key, slot);
            (slot, generation, false)
        } else {
            let slot = table.entries.len() as u32;
            table.entries.push(SlotEntry {
                key: slot_key,
                generation: 0,
                value: Some(cell(value)),
                last_alive_epoch: epoch,
            });
            table.index_slot(slot_key, slot);
            (slot, 0, false)
        }
    });
    if changed {
        for instance_key in context_read_subscribers(slot, generation) {
            record_replay_boundary_invalidation_for_instance_key(instance_key);
        }
    }

    let map = current_context_map().update(
        type_id,
        ContextSnapshotEntry {
            slot,
            generation,
            key: slot_key,
        },
    );
    with_context_map(map, f)
}

/// Reads inherited visual defaults from the current build scope.
///
/// Returns `None` when no provider of this type is present.
pub fn use_context<T>() -> Option<Context<T>>
where
    T: 'static,
{
    ensure_build_phase();

    with_execution_context(|context| {
        context
            .context_map
            .get(&TypeId::of::<T>())
            .copied()
            .map(|entry| Context::new(entry.slot, entry.generation))
    })
}
// (legacy comment removed)

#[cfg(test)]
mod tests {
    use std::{any::TypeId, cell::RefCell, rc::Rc};

    use super::{
        ContextMap, ContextSnapshotEntry, SlotEntry, SlotKey, SlotTable,
        normalize_context_snapshot, with_slot_table_mut,
    };
    use crate::execution_context::{
        reset_execution_context, with_execution_context, with_execution_context_mut,
    };
    use crate::runtime::{GroupPath, RuntimePhase, push_phase, with_replay_scope};

    fn with_context_snapshot<R>(snapshot: &ContextMap, body: impl FnOnce() -> R) -> R {
        with_replay_scope(
            0,
            GroupPath::default(),
            None,
            normalize_context_snapshot(snapshot),
            body,
        )
    }

    fn reset_test_state() {
        with_slot_table_mut(|table| *table = SlotTable::default());
        reset_execution_context();
    }

    #[test]
    fn context_cleanup_drops_values_outside_table_borrow() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        struct ReadOnDrop;
        impl Drop for ReadOnDrop {
            fn drop(&mut self) {
                super::with_slot_table(|table| {
                    assert!(table.entries.iter().all(|entry| entry.value.is_none()));
                    assert!(table.key_to_slot.is_empty());
                });
            }
        }

        for cleanup in 0..3 {
            reset_test_state();
            let key = SlotKey {
                instance_logic_id: 42,
                slot_hash: 7,
                type_id: TypeId::of::<ReadOnDrop>(),
            };
            with_slot_table_mut(|table| {
                table.entries.push(SlotEntry {
                    key,
                    generation: 0,
                    value: Some(Rc::new(ReadOnDrop)),
                    last_alive_epoch: 0,
                });
                table.index_slot(key, 0);
                table.begin_epoch();
            });
            let instances = super::HashSet::from_iter([42]);
            if cleanup == 2 {
                crate::runtime::reset_slots();
            } else if cleanup == 1 {
                super::recycle_recomposed_context_slots_for_instance_logic_ids(&instances);
            } else {
                super::drop_context_slots_for_instance_logic_ids(&instances);
            }
            super::with_slot_table(|table| {
                assert_eq!(table.free_list, [0]);
                assert_eq!(table.entries[0].generation, 1);
            });
        }
    }

    #[test]
    fn with_context_snapshot_restores_bindings_after_panic() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_test_state();

        let base_layer = ContextMap::new();
        let parent_layer = base_layer.update(
            TypeId::of::<u8>(),
            ContextSnapshotEntry {
                slot: 1,
                generation: 2,
                key: SlotKey {
                    instance_logic_id: 7,
                    slot_hash: 11,
                    type_id: TypeId::of::<u8>(),
                },
            },
        );
        with_execution_context_mut(|context| {
            context.context_map = parent_layer.clone();
        });

        let snapshot = ContextMap::new().update(
            TypeId::of::<u32>(),
            ContextSnapshotEntry {
                slot: 3,
                generation: 4,
                key: SlotKey {
                    instance_logic_id: 17,
                    slot_hash: 19,
                    type_id: TypeId::of::<u32>(),
                },
            },
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_context_snapshot(&snapshot, || {
                with_execution_context(|context| {
                    assert!(context.context_map.is_empty());
                });
                panic!("expected panic in context snapshot test");
            });
        }));
        assert!(result.is_err());

        with_execution_context(|context| {
            assert_eq!(context.context_map, parent_layer);
        });
    }

    #[test]
    fn with_context_snapshot_shares_live_bindings() {
        #[crate::tessera]
        fn context_provider() {
            super::provide_context(42_u8, || {
                let snapshot = super::current_context_map();
                with_context_snapshot(&snapshot, || {
                    assert!(snapshot.ptr_eq(&super::current_context_map()));
                });
            });
        }

        crate::testing::layout_test(|| context_provider()).run();
    }

    #[test]
    fn with_context_snapshot_remaps_stale_generation() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_test_state();

        let key = SlotKey {
            instance_logic_id: 31,
            slot_hash: 37,
            type_id: TypeId::of::<u8>(),
        };
        {
            with_slot_table_mut(|table| {
                *table = SlotTable::default();
                table.entries.push(SlotEntry {
                    key,
                    generation: 2,
                    value: Some(Rc::new(RefCell::new(42_u8))),
                    last_alive_epoch: 1,
                });
                table.index_slot(key, 0);
            });
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
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_test_state();

        let key = SlotKey {
            instance_logic_id: 41,
            slot_hash: 43,
            type_id: TypeId::of::<u16>(),
        };
        let old_key = SlotKey {
            instance_logic_id: 47,
            slot_hash: 53,
            type_id: TypeId::of::<u16>(),
        };
        {
            with_slot_table_mut(|table| {
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
                    value: Some(Rc::new(RefCell::new(7_u16))),
                    last_alive_epoch: 2,
                });
                table.index_slot(key, 1);
            });
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
