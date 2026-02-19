pub mod router;
pub mod task_handles;
mod tokio_runtime;

use std::{
    any::{Any, type_name},
    hash::Hash,
    sync::Arc,
};

use dashmap::{DashMap, mapref::entry::Entry};

/// Trait for shard state that can be auto-injected into `shard component`.
pub trait ShardState: Any + Send + Sync {}

/// Describes the lifecycle of this ShardState.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ShardStateLifeCycle {
    /// State exists for the lifetime of a router scope.
    Scope,
    /// State exists for the lifetime of a route instance.
    Shard,
}

impl<T> ShardState for T where T: 'static + Send + Sync + Default {}

pub(crate) type ErasedShardState = dyn Any + Send + Sync;
pub(crate) type ErasedShardStateHandle = Arc<ErasedShardState>;
pub(crate) type ShardStateMap<K> = DashMap<K, ErasedShardStateHandle>;

pub(crate) fn init_or_get_shard_state_in_map<K, T, F, R>(
    map: &ShardStateMap<K>,
    key: K,
    shard_id: &str,
    storage_label: &str,
    f: F,
) -> R
where
    K: Eq + Hash,
    T: ShardState + Default + 'static,
    F: FnOnce(Arc<T>) -> R,
{
    let erased = match map.entry(key) {
        Entry::Occupied(entry) => entry.get().clone(),
        Entry::Vacant(entry) => {
            let value: ErasedShardStateHandle = Arc::new(T::default());
            entry.insert(value.clone());
            value
        }
    };

    let typed = Arc::downcast::<T>(erased).unwrap_or_else(|_| {
        panic!(
            "shard state type mismatch for `{}` in {} storage: expected {}",
            shard_id,
            storage_label,
            type_name::<T>()
        )
    });
    f(typed)
}
