use std::{
    any::{TypeId, type_name},
    hash::Hash,
    marker::PhantomData,
    sync::Arc,
};

use parking_lot::RwLock;

use crate::state::{
    ShardStateSlot,
    storage::{ErasedShardStateHandle, shard_slot_table},
};

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

impl<T> Hash for ShardState<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.slot.hash(state);
        self.generation.hash(state);
    }
}

impl<T> ShardState<T> {
    pub(crate) fn from_slot(slot: ShardStateSlot) -> Self {
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
