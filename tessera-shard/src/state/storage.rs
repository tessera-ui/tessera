use std::{
    any::{Any, TypeId, type_name},
    hash::Hash,
    sync::{
        Arc, Weak,
        atomic::{AtomicU64, Ordering},
    },
};

use dashmap::{DashMap, mapref::entry::Entry};
use parking_lot::RwLock;

use crate::state::handle::ShardState;

pub(crate) type ErasedShardState = dyn Any + Send + Sync;
pub(crate) type ErasedShardStateHandle = Arc<ErasedShardState>;

/// Slot identity inside one controller's table.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ShardStateSlot {
    pub(crate) controller: u64,
    pub(crate) slot: u32,
    pub(crate) generation: u64,
}

pub(crate) struct ShardSlotEntry {
    pub(crate) generation: u64,
    pub(crate) type_id: TypeId,
    pub(crate) value: Option<ErasedShardStateHandle>,
}

pub(crate) struct ShardSlotTable {
    pub(crate) controller_id: u64,
    pub(crate) entries: Vec<ShardSlotEntry>,
    pub(crate) free_list: Vec<u32>,
}

/// Slot table owned by one [`crate::router::RouterController`].
///
/// Values are stored directly. There is no per-value lock.
pub(crate) struct ShardSlotTableHandle {
    id: u64,
    table: Arc<RwLock<ShardSlotTable>>,
}

static NEXT_CONTROLLER_ID: AtomicU64 = AtomicU64::new(1);

impl ShardSlotTableHandle {
    pub(crate) fn new() -> Self {
        let id = NEXT_CONTROLLER_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id,
            table: Arc::new(RwLock::new(ShardSlotTable {
                controller_id: id,
                entries: Vec::new(),
                free_list: Vec::new(),
            })),
        }
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn downgrade(&self) -> Weak<RwLock<ShardSlotTable>> {
        Arc::downgrade(&self.table)
    }
}

impl Clone for ShardSlotTableHandle {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            table: Arc::clone(&self.table),
        }
    }
}

fn alloc_shard_state_slot<T>(table_handle: &ShardSlotTableHandle) -> ShardStateSlot
where
    T: Default + Send + Sync + 'static,
{
    let value: ErasedShardStateHandle = Arc::new(T::default());
    let mut table = table_handle.table.write();
    let type_id = TypeId::of::<T>();

    if let Some(slot) = table.free_list.pop() {
        let entry = table
            .entries
            .get_mut(slot as usize)
            .expect("shard slot entry should exist");
        entry.type_id = type_id;
        entry.value = Some(value);
        return ShardStateSlot {
            controller: table_handle.id,
            slot,
            generation: entry.generation,
        };
    }

    let slot = table.entries.len() as u32;
    table.entries.push(ShardSlotEntry {
        generation: 0,
        type_id,
        value: Some(value),
    });
    ShardStateSlot {
        controller: table_handle.id,
        slot,
        generation: 0,
    }
}

fn assert_slot_type_for<T>(
    table_handle: &ShardSlotTableHandle,
    slot: ShardStateSlot,
    shard_id: &str,
    storage_label: &str,
) where
    T: Send + Sync + 'static,
{
    let table = table_handle.table.read();
    let entry = table.entries.get(slot.slot as usize).unwrap_or_else(|| {
        panic!(
            "shard state slot {} for `{}` in {} storage is missing",
            slot.slot, shard_id, storage_label
        )
    });

    if slot.controller != table_handle.id || entry.generation != slot.generation {
        panic!(
            "shard state for `{}` in {} storage is stale (controller {}, slot {}, generation {}, current generation {})",
            shard_id, storage_label, slot.controller, slot.slot, slot.generation, entry.generation
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

pub(crate) fn recycle_shard_state_slot(table_handle: &ShardSlotTableHandle, slot: ShardStateSlot) {
    if slot.controller != table_handle.id {
        return;
    }
    let mut table = table_handle.table.write();
    let Some(entry) = table.entries.get_mut(slot.slot as usize) else {
        return;
    };
    if entry.generation != slot.generation {
        return;
    }
    let value = entry.value.take();
    entry.generation = entry.generation.wrapping_add(1);
    table.free_list.push(slot.slot);
    drop(table);
    drop(value);
}

pub(crate) type ShardStateMap<K> = DashMap<K, ShardStateSlot>;

pub(crate) fn init_or_get_shard_state_in_map<K, T, F, R>(
    table_handle: &ShardSlotTableHandle,
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
            let value = alloc_shard_state_slot::<T>(table_handle);
            entry.insert(value);
            value
        }
    };

    assert_slot_type_for::<T>(table_handle, slot, shard_id, storage_label);
    f(ShardState::from_slot(table_handle, slot))
}

#[cfg(test)]
mod tests {
    use super::*;

    std::thread_local! {
        static ACTIVE_TABLE: std::cell::RefCell<Option<ShardSlotTableHandle>> =
            const { std::cell::RefCell::new(None) };
    }

    struct ReentrantState;

    impl Default for ReentrantState {
        fn default() -> Self {
            ACTIVE_TABLE.with(|cell| {
                let tables = cell.borrow();
                let tables = tables.as_ref().expect("table under test");
                let nested = alloc_shard_state_slot::<u8>(tables);
                recycle_shard_state_slot(tables, nested);
            });
            Self
        }
    }

    impl Drop for ReentrantState {
        fn drop(&mut self) {
            ACTIVE_TABLE.with(|cell| {
                let tables = cell.borrow();
                let tables = tables.as_ref().expect("table under test");
                let nested = alloc_shard_state_slot::<u8>(tables);
                recycle_shard_state_slot(tables, nested);
            });
        }
    }

    #[test]
    fn user_initializers_and_destructors_run_outside_table_lock() {
        let tables = ShardSlotTableHandle::new();
        ACTIVE_TABLE.with(|cell| {
            *cell.borrow_mut() = Some(tables.clone());
        });
        let slot = alloc_shard_state_slot::<ReentrantState>(&tables);
        recycle_shard_state_slot(&tables, slot);
        recycle_shard_state_slot(&tables, slot);
        {
            let table = tables.table.read();
            assert_ne!(
                table.entries[slot.slot as usize].generation,
                slot.generation
            );
            assert!(
                table
                    .free_list
                    .iter()
                    .filter(|&&free| free == slot.slot)
                    .count()
                    <= 1
            );
        }
        let next = alloc_shard_state_slot::<ReentrantState>(&tables);
        recycle_shard_state_slot(&tables, next);
        ACTIVE_TABLE.with(|cell| {
            *cell.borrow_mut() = None;
        });
    }
}
