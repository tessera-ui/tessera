use std::sync::Arc;

use tessera_ui::{State, StateOwner};

use crate::{RouterDestination, ShardState, ShardStateLifeCycle};

struct RouteEntry {
    destination: Arc<dyn RouterDestination>,
    owner: StateOwner,
}

/// Owns the destination stack and reactive state for its routes.
pub struct RouterController {
    route_stack: Vec<RouteEntry>,
    scope: StateOwner,
    marker: State<()>,
}

impl RouterController {
    /// Creates an empty controller in the active UI session.
    pub fn new() -> Self {
        let scope = StateOwner::new();
        let marker = scope.state_with_key("controller", || ());
        Self {
            route_stack: Vec::new(),
            scope,
            marker,
        }
    }

    /// Creates a controller seeded with a root destination.
    pub fn with_root(root_dest: impl RouterDestination + 'static) -> Self {
        let mut router = Self::new();
        router.push(root_dest);
        router
    }

    /// Creates a controller seeded with a shared root destination.
    pub fn with_root_shared(root_dest: Arc<dyn RouterDestination>) -> Self {
        let mut router = Self::new();
        router.push_shared(root_dest);
        router
    }

    /// Pushes a destination with its own state lifetime.
    pub fn push<T: RouterDestination + 'static>(&mut self, destination: T) {
        self.push_shared(Arc::new(destination));
    }

    /// Pushes a shared destination with its own state lifetime.
    pub fn push_shared(&mut self, destination: Arc<dyn RouterDestination>) {
        self.route_stack.push(RouteEntry {
            destination,
            owner: StateOwner::new(),
        });
    }

    /// Pops the current destination and releases its route state.
    pub fn pop(&mut self) -> Option<Arc<dyn RouterDestination>> {
        let removed = self.route_stack.pop()?;
        removed.owner.close();
        Some(removed.destination)
    }

    /// Replaces the current destination, or pushes into an empty stack.
    pub fn replace<T: RouterDestination + 'static>(
        &mut self,
        destination: T,
    ) -> Option<Arc<dyn RouterDestination>> {
        let previous = self.pop();
        self.push(destination);
        previous
    }

    /// Replaces the current destination with a shared destination.
    pub fn replace_shared(
        &mut self,
        destination: Arc<dyn RouterDestination>,
    ) -> Option<Arc<dyn RouterDestination>> {
        let previous = self.pop();
        self.push_shared(destination);
        previous
    }

    /// Returns whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.route_stack.is_empty()
    }
    /// Returns the number of destinations.
    pub fn len(&self) -> usize {
        self.route_stack.len()
    }
    /// Returns the current destination.
    pub fn last(&self) -> Option<&dyn RouterDestination> {
        self.route_stack.last().map(|entry| &*entry.destination)
    }
    /// Returns whether the current destination has type `D`.
    pub fn current_is<D: RouterDestination + 'static>(&self) -> bool {
        self.last()
            .is_some_and(|current| current.type_id() == std::any::TypeId::of::<D>())
    }

    pub(crate) fn current_destination(
        &self,
    ) -> Option<Arc<dyn Fn(State<RouterController>) + Send + Sync>> {
        self.route_stack
            .last()
            .map(|entry| entry.destination.exec_component_handle())
    }

    pub(crate) fn state_context(&self, lifecycle: ShardStateLifeCycle) -> (StateOwner, State<()>) {
        let owner = match lifecycle {
            ShardStateLifeCycle::Scope => &self.scope,
            ShardStateLifeCycle::Shard => {
                &self
                    .route_stack
                    .last()
                    .expect("route-scoped shard state requires a non-empty router stack")
                    .owner
            }
        };
        (owner.clone(), self.marker)
    }

    /// Gets or initializes state owned by the current route.
    pub fn init_or_get<T, F, R>(&self, id: &str, f: F) -> R
    where
        T: Default + Send + Sync + 'static,
        F: FnOnce(ShardState<T>) -> R,
    {
        self.init_or_get_with_lifecycle(id, ShardStateLifeCycle::Shard, f)
    }

    /// Gets or initializes state owned by a route or this controller.
    pub fn init_or_get_with_lifecycle<T, F, R>(
        &self,
        id: &str,
        lifecycle: ShardStateLifeCycle,
        f: F,
    ) -> R
    where
        T: Default + Send + Sync + 'static,
        F: FnOnce(ShardState<T>) -> R,
    {
        let (owner, marker) = self.state_context(lifecycle);
        f(ShardState::new(
            owner.state_with_key(("shard", id), T::default),
            marker,
        ))
    }

    /// Clears all destinations and releases their route state.
    pub fn clear(&mut self) {
        self.route_stack.clear();
    }
    /// Clears the stack and installs a new root destination.
    pub fn reset(&mut self, root_dest: impl RouterDestination + 'static) {
        self.clear();
        self.push(root_dest);
    }
    /// Clears the stack and installs a shared root destination.
    pub fn reset_shared(&mut self, root_dest: Arc<dyn RouterDestination>) {
        self.clear();
        self.push_shared(root_dest);
    }
}

impl Default for RouterController {
    fn default() -> Self {
        Self::new()
    }
}
impl Drop for RouterController {
    fn drop(&mut self) {
        self.route_stack.clear();
        self.scope.close();
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::RouterController;
    use crate::{RouterDestination, ShardStateLifeCycle};

    #[derive(Default)]
    struct CounterState {
        value: usize,
    }

    struct DummyDestination;

    impl RouterDestination for DummyDestination {
        fn exec_component(&self, _: tessera_ui::State<RouterController>) {}

        fn exec_component_handle(
            &self,
        ) -> std::sync::Arc<dyn Fn(tessera_ui::State<RouterController>) + Send + Sync> {
            std::sync::Arc::new(|_| {})
        }

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
            state.with_mut(|value| {
                value.value += 1;
                value.value
            })
        })
    }

    #[test]
    fn route_scoped_state_is_released_on_pop() {
        let session = tessera_ui::RuntimeSession::new();
        let _session = session.enter();
        let shard_id = "route_scoped";
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
        let session = tessera_ui::RuntimeSession::new();
        let _session = session.enter();
        let shard_id = "scope_scoped";
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
        let session = tessera_ui::RuntimeSession::new();
        let _session = session.enter();
        let shard_id = "route_context_required";
        let router = RouterController::new();
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _ = increment_state(&router, shard_id, ShardStateLifeCycle::Shard);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn controllers_do_not_share_or_alias_slots() {
        let session = tessera_ui::RuntimeSession::new();
        let _session = session.enter();
        let shard_id = "isolated";
        let left = RouterController::with_root(DummyDestination);
        let right = RouterController::with_root(DummyDestination);

        left.init_or_get::<CounterState, _, _>(shard_id, |state| {
            state.with_mut(|value| value.value = 11);
        });
        right.init_or_get::<CounterState, _, _>(shard_id, |state| {
            state.with_mut(|value| value.value = 22);
        });

        assert_eq!(
            left.init_or_get::<CounterState, _, _>(shard_id, |state| {
                state.with(|value| value.value)
            }),
            11
        );
        assert_eq!(
            right.init_or_get::<CounterState, _, _>(shard_id, |state| {
                state.with(|value| value.value)
            }),
            22
        );

        let stale = left.init_or_get::<CounterState, _, _>(shard_id, |state| state);
        drop(left);

        right.init_or_get::<CounterState, _, _>(shard_id, |state| {
            state.with_mut(|value| value.value = 33);
        });
        assert!(stale.try_with(|value| value.value).is_err());
        assert_eq!(
            right.init_or_get::<CounterState, _, _>(shard_id, |state| {
                state.with(|value| value.value)
            }),
            33
        );
    }
    #[test]
    fn panicking_mutation_restores_the_live_shard_value() {
        let session = tessera_ui::RuntimeSession::new();
        let _session = session.enter();
        let router = RouterController::with_root(DummyDestination);
        let state = router.init_or_get::<usize, _, _>("panic", |state| state);
        let result = catch_unwind(AssertUnwindSafe(|| {
            state.with_mut(|value| {
                *value = 42;
                panic!("abort mutation");
            });
        }));
        assert!(result.is_err());
        assert_eq!(state.get(), 42);
        state.set(43);
        assert_eq!(state.get(), 43);
    }
    type Observed = (
        tessera_ui::State<RouterController>,
        crate::ShardState<usize>,
        usize,
    );

    #[tessera_ui::tessera]
    fn reactive_fixture(output: tessera_ui::testing::TestProbe<Option<Observed>>) {
        let router = tessera_ui::remember(|| RouterController::with_root(DummyDestination));
        let state = crate::router::with_current_router_shard_state(
            "reactive",
            ShardStateLifeCycle::Shard,
            router,
            |state: crate::ShardState<usize>| state,
        );
        output.set(Some((router, state, state.get())));
    }

    #[test]
    fn shard_writes_rebuild_consumers_and_route_owners_preserve_only_live_state() {
        use tessera_ui::testing::{
            TestProbe, layout_test, pump_layout_test_session, start_layout_test_session,
        };

        let output = TestProbe::default();
        let probe = output.clone();
        let mut session = start_layout_test_session(layout_test(move || {
            reactive_fixture(probe.clone());
        }));
        pump_layout_test_session(&mut session, 0);
        let (router, root_state, value) = output.get().unwrap();
        assert_eq!(value, 0);
        pump_layout_test_session(&mut session, 1);
        assert_eq!(
            session
                .work()
                .count("reactive_fixture", tessera_ui::WorkPhase::Build),
            0
        );
        session.run(|| root_state.set(7));
        pump_layout_test_session(&mut session, 2);
        assert_eq!(output.get().unwrap().2, 7);
        assert_eq!(
            session
                .work()
                .count("reactive_fixture", tessera_ui::WorkPhase::Build),
            1
        );

        session.run(|| router.with_mut(|router| router.push(DummyDestination)));
        pump_layout_test_session(&mut session, 3);
        let (_, top_state, value) = output.get().unwrap();
        assert_eq!(value, 0);
        assert!(top_state != root_state);
        session.run(|| top_state.set(12));
        pump_layout_test_session(&mut session, 4);
        assert_eq!(output.get().unwrap().2, 12);
        session.run(|| {
            router.with_mut(|router| {
                router.pop();
            })
        });
        pump_layout_test_session(&mut session, 5);
        assert!(output.get().unwrap().1 == root_state);
        assert_eq!(output.get().unwrap().2, 7);
        assert_eq!(
            session.run(|| top_state.try_with(|value| *value)),
            Err(crate::state::ShardStateError::Stale)
        );
        session.run(|| {
            router.with_mut(|router| {
                router.clear();
            })
        });
        assert_eq!(
            session.run(|| root_state.try_with(|value| *value)),
            Err(crate::state::ShardStateError::Stale)
        );
    }
}
