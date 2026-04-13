use std::{
    any::{Any, TypeId, type_name},
    hash::Hash,
    sync::{Arc, OnceLock},
};

use dashmap::{DashMap, mapref::entry::Entry};
use parking_lot::RwLock;

use crate::state::handle::ShardState;

pub(crate) type ErasedShardState = dyn Any + Send + Sync;
pub(crate) type ErasedShardStateHandle = Arc<ErasedShardState>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ShardStateSlot {
    pub(crate) slot: u32,
    pub(crate) generation: u64,
}

pub(crate) struct ShardSlotEntry {
    pub(crate) generation: u64,
    pub(crate) type_id: TypeId,
    pub(crate) value: Option<ErasedShardStateHandle>,
}

#[derive(Default)]
pub(crate) struct ShardSlotTable {
    pub(crate) entries: Vec<ShardSlotEntry>,
    pub(crate) free_list: Vec<u32>,
}

static SHARD_SLOT_TABLE: OnceLock<RwLock<ShardSlotTable>> = OnceLock::new();

pub(crate) fn shard_slot_table() -> &'static RwLock<ShardSlotTable> {
    SHARD_SLOT_TABLE.get_or_init(|| RwLock::new(ShardSlotTable::default()))
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
