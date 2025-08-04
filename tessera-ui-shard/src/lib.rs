use std::{any::Any, sync::OnceLock};

use dashmap::DashMap;

trait AsAny {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

static REGISTRY: OnceLock<ShardRegistry> = OnceLock::new();

/// Trait for shard state that can be auto-injected into `shard component`.
pub trait ShardState: Any + Send + Sync {}

impl<T> ShardState for T where T: Any + Send + Sync + Default {}

pub struct ShardRegistry {
    shards: DashMap<String, Box<dyn ShardState>>,
}

impl ShardRegistry {
    /// Get the singleton instance of the shard registry.
    ///
    /// Should only be called by macro, not manually.
    pub fn get() -> &'static Self {
        REGISTRY.get_or_init(|| ShardRegistry {
            shards: DashMap::new(),
        })
    }

    pub fn with_mut_or_init<T, F, R>(&self, id: &str, f: F) -> R
    where
        T: ShardState + Default + 'static,
        F: FnOnce(&mut T) -> R,
    {
        let mut shard_ref = self
            .shards
            .entry(id.to_string())
            .or_insert_with(|| Box::new(T::default()));

        // a. shard_ref.value_mut() returns &mut Box<dyn ShardState>
        //    Through DerefMut, we get &mut dyn ShardState
        let trait_object = shard_ref.value_mut();

        // b. Call our defined as_any_mut.
        //    The compiler knows that the lifetime of `trait_object` here is `'1`,
        //    and `as_any_mut` will return a `&'1 mut dyn Any` with the same lifetime.
        //    This explicit conversion step helps the compiler's lifetime inference.
        let any_mut = trait_object.as_any_mut();

        // c. Now calling downcast_mut on &'1 mut dyn Any is clear.
        //    It will return an Option<&'1 mut T>, with the correct lifetime.
        if let Some(state) = any_mut.downcast_mut::<T>() {
            f(state)
        } else {
            // This branch should theoretically never be triggered, since we are get-or-insert
            panic!("Shard with id '{}' has an unexpected type.", id);
        }
    }
}
