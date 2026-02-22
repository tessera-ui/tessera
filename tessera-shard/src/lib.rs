pub mod router;
pub mod task_handles;
mod tokio_runtime;

use std::{
    any::{Any, TypeId, type_name},
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, OnceLock},
};

use dashmap::{DashMap, mapref::entry::Entry};
use parking_lot::RwLock;

/// Describes the lifecycle of this shard state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ShardStateLifeCycle {
    /// State exists for the lifetime of a router scope.
    Scope,
    /// State exists for the lifetime of a route instance.
    Shard,
}

pub(crate) type ErasedShardState = dyn Any + Send + Sync;
pub(crate) type ErasedShardStateHandle = Arc<ErasedShardState>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ShardStateSlot {
    slot: u32,
    generation: u64,
}

struct ShardSlotEntry {
    generation: u64,
    type_id: TypeId,
    value: Option<ErasedShardStateHandle>,
}

#[derive(Default)]
struct ShardSlotTable {
    entries: Vec<ShardSlotEntry>,
    free_list: Vec<u32>,
}

static SHARD_SLOT_TABLE: OnceLock<RwLock<ShardSlotTable>> = OnceLock::new();

fn shard_slot_table() -> &'static RwLock<ShardSlotTable> {
    SHARD_SLOT_TABLE.get_or_init(|| RwLock::new(ShardSlotTable::default()))
}

/// Typed shard state handle.
///
/// This is a lightweight `Copy` handle backed by shard-managed storage.
/// The actual value is hosted in a global slot table and validated via
/// generation to prevent ABA stale access.
pub struct ShardState<T> {
    slot: u32,
    generation: u64,
    _marker: PhantomData<T>,
}

impl<T> Copy for ShardState<T> {}

impl<T> Clone for ShardState<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for ShardState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot && self.generation == other.generation
    }
}

impl<T> Eq for ShardState<T> {}

impl<T> std::hash::Hash for ShardState<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.slot.hash(state);
        self.generation.hash(state);
    }
}

impl<T> ShardState<T> {
    fn from_slot(slot: ShardStateSlot) -> Self {
        Self {
            slot: slot.slot,
            generation: slot.generation,
            _marker: PhantomData,
        }
    }
}

impl<T> ShardState<T>
where
    T: Send + Sync + 'static,
{
    fn load_entry(&self) -> ErasedShardStateHandle {
        let table = shard_slot_table().read();
        let entry = table
            .entries
            .get(self.slot as usize)
            .unwrap_or_else(|| panic!("ShardState points to freed slot: {}", self.slot));

        if entry.generation != self.generation {
            panic!(
                "ShardState is stale (slot {}, generation {}, current generation {})",
                self.slot, self.generation, entry.generation
            );
        }

        if entry.type_id != TypeId::of::<T>() {
            panic!(
                "ShardState type mismatch for slot {}: expected {}, stored {:?}",
                self.slot,
                type_name::<T>(),
                entry.type_id
            );
        }

        entry
            .value
            .as_ref()
            .unwrap_or_else(|| panic!("ShardState slot {} has been recycled", self.slot))
            .clone()
    }

    fn load_lock(&self) -> Arc<RwLock<T>> {
        self.load_entry()
            .downcast::<RwLock<T>>()
            .unwrap_or_else(|_| panic!("ShardState slot {} downcast failed", self.slot))
    }

    /// Execute a closure with a shared reference to the value.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let lock = self.load_lock();
        let guard = lock.read();
        f(&guard)
    }

    /// Execute a closure with a mutable reference to the value.
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let lock = self.load_lock();
        let mut guard = lock.write();
        f(&mut guard)
    }

    /// Get a cloned value. Requires `T: Clone`.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with(Clone::clone)
    }

    /// Replace the stored value.
    pub fn set(&self, value: T) {
        self.with_mut(|slot| *slot = value);
    }
}

fn alloc_shard_state_slot<T>() -> ShardStateSlot
where
    T: Default + Send + Sync + 'static,
{
    let mut table = shard_slot_table().write();
    let type_id = TypeId::of::<T>();

    if let Some(slot) = table.free_list.pop() {
        let entry = table
            .entries
            .get_mut(slot as usize)
            .expect("shard slot entry should exist");
        entry.type_id = type_id;
        entry.value = Some(Arc::new(RwLock::new(T::default())));
        return ShardStateSlot {
            slot,
            generation: entry.generation,
        };
    }

    let slot = table.entries.len() as u32;
    table.entries.push(ShardSlotEntry {
        generation: 0,
        type_id,
        value: Some(Arc::new(RwLock::new(T::default()))),
    });
    ShardStateSlot {
        slot,
        generation: 0,
    }
}

fn assert_slot_type_for<T>(slot: ShardStateSlot, shard_id: &str, storage_label: &str)
where
    T: Send + Sync + 'static,
{
    let table = shard_slot_table().read();
    let entry = table.entries.get(slot.slot as usize).unwrap_or_else(|| {
        panic!(
            "shard state slot {} for `{}` in {} storage is missing",
            slot.slot, shard_id, storage_label
        )
    });

    if entry.generation != slot.generation {
        panic!(
            "shard state for `{}` in {} storage is stale (slot {}, generation {}, current generation {})",
            shard_id, storage_label, slot.slot, slot.generation, entry.generation
        );
    }

    if entry.value.is_none() {
        panic!(
            "shard state for `{}` in {} storage has been recycled (slot {})",
            shard_id, storage_label, slot.slot
        );
    }

    if entry.type_id != TypeId::of::<T>() {
        panic!(
            "shard state type mismatch for `{}` in {} storage: expected {}",
            shard_id,
            storage_label,
            type_name::<T>()
        );
    }
}

pub(crate) fn recycle_shard_state_slot(slot: ShardStateSlot) {
    let mut table = shard_slot_table().write();
    let Some(entry) = table.entries.get_mut(slot.slot as usize) else {
        return;
    };
    if entry.generation != slot.generation {
        return;
    }
    entry.value = None;
    entry.generation = entry.generation.wrapping_add(1);
    table.free_list.push(slot.slot);
}

pub(crate) type ShardStateMap<K> = DashMap<K, ShardStateSlot>;

pub(crate) fn init_or_get_shard_state_in_map<K, T, F, R>(
    map: &ShardStateMap<K>,
    key: K,
    shard_id: &str,
    storage_label: &str,
    f: F,
) -> R
where
    K: Eq + Hash,
    T: Default + Send + Sync + 'static,
    F: FnOnce(ShardState<T>) -> R,
{
    let slot = match map.entry(key) {
        Entry::Occupied(entry) => *entry.get(),
        Entry::Vacant(entry) => {
            let value = alloc_shard_state_slot::<T>();
            entry.insert(value);
            value
        }
    };

    assert_slot_type_for::<T>(slot, shard_id, storage_label);
    f(ShardState::from_slot(slot))
}
