use std::{collections::HashSet, sync::Arc};

use crate::{
    router::{RouteId, RouteShardKey, RouterDestination},
    state::{
        ShardState, ShardStateLifeCycle, ShardStateMap, init_or_get_shard_state_in_map,
        recycle_shard_state_slot,
    },
};

struct RouteEntry {
    route_id: RouteId,
    destination: Arc<dyn RouterDestination>,
}

/// Reactive navigation controller for one shard tree.
pub struct RouterController {
    route_stack: Vec<RouteEntry>,
    scope_shards: ShardStateMap<String>,
    route_shards: ShardStateMap<RouteShardKey>,
}

impl RouterController {
    /// Create an empty controller.
    pub fn new() -> Self {
        Self {
            route_stack: Vec::new(),
            scope_shards: Default::default(),
            route_shards: Default::default(),
        }
    }

    /// Create a controller seeded with a root destination.
    pub fn with_root(root_dest: impl RouterDestination + 'static) -> Self {
        let mut router = Self::new();
        router.push(root_dest);
        router
    }

    /// Create a controller seeded with a shared root destination.
    pub fn with_root_shared(root_dest: Arc<dyn RouterDestination>) -> Self {
        let mut router = Self::new();
        router.push_shared(root_dest);
        router
    }

    /// Push a destination onto the stack.
    pub fn push<T: RouterDestination + 'static>(&mut self, destination: T) {
        self.push_shared(Arc::new(destination));
    }

    /// Push a shared destination onto the stack.
    pub fn push_shared(&mut self, destination: Arc<dyn RouterDestination>) {
        self.route_stack.push(RouteEntry {
            route_id: RouteId::new(),
            destination,
        });
    }

    /// Pop the top destination from the stack.
    ///
    /// Returns `None` if the stack is empty.
    pub fn pop(&mut self) -> Option<Arc<dyn RouterDestination>> {
        let removed = self.route_stack.pop()?;
        self.prune_route_shards(removed.route_id);
        Some(removed.destination)
    }

    /// Replace the top destination.
    ///
    /// If the stack is empty, this behaves like [`Self::push`].
    pub fn replace<T: RouterDestination + 'static>(
        &mut self,
        destination: T,
    ) -> Option<Arc<dyn RouterDestination>> {
        let previous = self.pop();
        self.push(destination);
        previous
    }

    /// Replace the top destination with a shared destination.
    pub fn replace_shared(
        &mut self,
        destination: Arc<dyn RouterDestination>,
    ) -> Option<Arc<dyn RouterDestination>> {
        let previous = self.pop();
        self.push_shared(destination);
        previous
    }

    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.route_stack.is_empty()
    }

    /// Number of destinations in the stack.
    pub fn len(&self) -> usize {
        self.route_stack.len()
    }

    /// Top destination.
    pub fn last(&self) -> Option<&dyn RouterDestination> {
        self.route_stack.last().map(|entry| &*entry.destination)
    }

    /// Whether the current destination matches `D`.
    pub fn current_is<D>(&self) -> bool
    where
        D: RouterDestination + 'static,
    {
        self.last()
            .is_some_and(|current| current.type_id() == std::any::TypeId::of::<D>())
    }

    pub(crate) fn current_route_id(&self) -> Option<RouteId> {
        self.route_stack.last().map(|entry| entry.route_id)
    }

    pub(crate) fn exec_current(&self) -> bool {
        let Some(entry) = self.route_stack.last() else {
            return false;
        };
        entry.destination.exec_component();
        true
    }

    /// Get or initialize route-scoped state and provide it to `f`.
    pub fn init_or_get<T, F, R>(&self, id: &str, f: F) -> R
    where
        T: Default + Send + Sync + 'static,
        F: FnOnce(ShardState<T>) -> R,
    {
        self.init_or_get_with_lifecycle(id, ShardStateLifeCycle::Shard, f)
    }

    /// Get or initialize state for a lifecycle scope and provide it to `f`.
    pub fn init_or_get_with_lifecycle<T, F, R>(
        &self,
        id: &str,
        life_cycle: ShardStateLifeCycle,
        f: F,
    ) -> R
    where
        T: Default + Send + Sync + 'static,
        F: FnOnce(ShardState<T>) -> R,
    {
        match life_cycle {
            ShardStateLifeCycle::Scope => {
                init_or_get_shard_state_in_map(&self.scope_shards, id.to_owned(), id, "scope", f)
            }
            ShardStateLifeCycle::Shard => {
                let route_id = self.current_route_id().unwrap_or_else(|| {
                    panic!("route-scoped shard state requires a non-empty router stack")
                });
                init_or_get_shard_state_in_map(
                    &self.route_shards,
                    RouteShardKey {
                        route_id,
                        shard_id: id.to_owned(),
                    },
                    id,
                    "route",
                    f,
                )
            }
        }
    }

    /// Clear all destinations from the stack.
    pub fn clear(&mut self) {
        if self.route_stack.is_empty() {
            return;
        }
        let removed_route_ids: HashSet<_> = self
            .route_stack
            .drain(..)
            .map(|entry| entry.route_id)
            .collect();
        let keys: Vec<_> = self
            .route_shards
            .iter()
            .filter(|entry| removed_route_ids.contains(&entry.key().route_id))
            .map(|entry| entry.key().clone())
            .collect();
        for key in keys {
            if let Some((_, slot)) = self.route_shards.remove(&key) {
                recycle_shard_state_slot(slot);
            }
        }
    }

    /// Clear all destinations and push a new root destination.
    pub fn reset(&mut self, root_dest: impl RouterDestination + 'static) {
        self.clear();
        self.push(root_dest);
    }

    /// Clear all destinations and push a shared root destination.
    pub fn reset_shared(&mut self, root_dest: Arc<dyn RouterDestination>) {
        self.clear();
        self.push_shared(root_dest);
    }

    fn prune_route_shards(&self, route_id: RouteId) {
        let keys: Vec<_> = self
            .route_shards
            .iter()
            .filter(|entry| entry.key().route_id == route_id)
            .map(|entry| entry.key().clone())
            .collect();
        for key in keys {
            if let Some((_, slot)) = self.route_shards.remove(&key) {
                recycle_shard_state_slot(slot);
            }
        }
    }
}

impl Default for RouterController {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for RouterController {
    fn drop(&mut self) {
        let scope_slots: Vec<_> = self
            .scope_shards
            .iter()
            .map(|entry| *entry.value())
            .collect();
        let route_slots: Vec<_> = self
            .route_shards
            .iter()
            .map(|entry| *entry.value())
            .collect();

        self.scope_shards.clear();
        self.route_shards.clear();

        for slot in scope_slots.into_iter().chain(route_slots) {
            recycle_shard_state_slot(slot);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        panic::{AssertUnwindSafe, catch_unwind},
        sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    };

    use super::RouterController;
    use crate::{RouterDestination, ShardStateLifeCycle};

    static TEST_SHARD_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

    fn unique_shard_id(prefix: &str) -> &'static str {
        let id = TEST_SHARD_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Box::leak(format!("{prefix}::{id}").into_boxed_str())
    }

    #[derive(Default)]
    struct CounterState {
        value: AtomicUsize,
    }

    struct DummyDestination;

    impl RouterDestination for DummyDestination {
        fn exec_component(&self) {}

        fn destination_id() -> &'static str {
            "dummy"
        }
    }

    fn increment_state(
        router: &RouterController,
        shard_id: &str,
        life_cycle: ShardStateLifeCycle,
    ) -> usize {
        router.init_or_get_with_lifecycle::<CounterState, _, _>(shard_id, life_cycle, |state| {
            state.with(|value| value.value.fetch_add(1, Ordering::SeqCst) + 1)
        })
    }

    #[test]
    fn route_scoped_state_is_released_on_pop() {
        let shard_id = unique_shard_id("route_scoped");
        let mut router = RouterController::with_root(DummyDestination);

        assert_eq!(
            increment_state(&router, shard_id, ShardStateLifeCycle::Shard),
            1
        );
        assert_eq!(
            increment_state(&router, shard_id, ShardStateLifeCycle::Shard),
            2
        );

        assert!(router.pop().is_some());
        router.push(DummyDestination);
        assert_eq!(
            increment_state(&router, shard_id, ShardStateLifeCycle::Shard),
            1
        );
    }

    #[test]
    fn scope_scoped_state_persists_inside_scope_but_resets_across_scopes() {
        let shard_id = unique_shard_id("scope_scoped");
        let mut router = RouterController::with_root(DummyDestination);

        assert_eq!(
            increment_state(&router, shard_id, ShardStateLifeCycle::Scope),
            1
        );

        router.push(DummyDestination);
        assert_eq!(
            increment_state(&router, shard_id, ShardStateLifeCycle::Scope),
            2
        );

        assert!(router.pop().is_some());
        assert_eq!(
            increment_state(&router, shard_id, ShardStateLifeCycle::Scope),
            3
        );

        drop(router);

        let router = RouterController::with_root(DummyDestination);
        assert_eq!(
            increment_state(&router, shard_id, ShardStateLifeCycle::Scope),
            1
        );
    }

    #[test]
    fn route_scoped_state_requires_active_route() {
        let shard_id = unique_shard_id("route_context_required");
        let router = RouterController::new();
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _ = increment_state(&router, shard_id, ShardStateLifeCycle::Shard);
        }));
        assert!(result.is_err());
    }
}
