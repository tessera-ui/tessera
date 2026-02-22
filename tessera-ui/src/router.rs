//! # Router
//!
//! Context-scoped routing utilities.
//!
//! Use `router_scope` to create a local router state and `router_view` to
//! render the current destination from that scope.
//!
//! `router_scope` seeds the stack with `root_dest` once when the scope state
//! is created. After that, stack mutations are explicit:
//!
//! - If the stack becomes empty (for example, via `pop` or `clear`),
//!   `router_view` will panic.
//! - There is no automatic root re-initialization.

use tessera_shard::router::{Router as ShardRouter, RouterScopeId};

pub use tessera_shard::router::RouterDestination;

use crate::{
    context::{context_from_previous_snapshot_for_instance, provide_context, use_context},
    runtime::{RuntimePhase, current_component_instance_key_in_scope, current_phase, remember},
};

#[derive(Clone, Copy)]
struct RouterContext {
    state: crate::State<ShardRouter>,
}

fn resolve_router_state() -> crate::State<ShardRouter> {
    match current_phase() {
        Some(RuntimePhase::Build) => {
            let context = use_context::<RouterContext>()
                .expect("Router is missing in build scope. Wrap UI with router_scope/router_root.");
            context.get().state
        }
        Some(RuntimePhase::Input) => {
            let instance_key = current_component_instance_key_in_scope()
                .expect("Router command requires an active component scope during input handling");
            let context = context_from_previous_snapshot_for_instance::<RouterContext>(
                instance_key,
            )
            .expect("Router is missing in input scope. Ensure callbacks run inside router_scope.");
            context.get().state
        }
        _ => {
            panic!("Router command must be called during build or input phase");
        }
    }
}

#[doc(hidden)]
pub fn with_current_router_shard_state<T, F, R>(
    shard_id: &str,
    life_cycle: tessera_shard::ShardStateLifeCycle,
    f: F,
) -> R
where
    T: Default + Send + Sync + 'static,
    F: FnOnce(tessera_shard::ShardState<T>) -> R,
{
    let state = resolve_router_state();
    state.with(|router| router.init_or_get_with_lifecycle(shard_id, life_cycle, f))
}

/// Scoped router access helper.
///
/// `Router` resolves to the nearest `router_scope`.
pub struct Router;

impl Router {
    fn with_router<F, R>(f: F) -> R
    where
        F: FnOnce(&ShardRouter) -> R,
    {
        let state = resolve_router_state();
        state.with(f)
    }

    fn with_router_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut ShardRouter) -> R,
    {
        let state = resolve_router_state();
        state.with_mut(f)
    }

    /// Push a destination on top of the current scoped stack.
    pub fn push(destination: impl RouterDestination + 'static) {
        Self::with_router_mut(|router| {
            router.push(destination);
        });
    }

    /// Pop the top destination.
    ///
    /// Returns `true` when a route was removed.
    pub fn pop() -> bool {
        Self::with_router_mut(|router| router.pop().is_some())
    }

    /// Replace the top destination.
    ///
    /// If the stack is empty, this behaves like [`Self::push`].
    pub fn replace(destination: impl RouterDestination + 'static) {
        Self::with_router_mut(|router| {
            let _ = router.replace(destination);
        });
    }

    /// Clear all destinations.
    pub fn clear() {
        Self::with_router_mut(ShardRouter::clear);
    }

    /// Reset to a single root destination.
    pub fn reset(root_dest: impl RouterDestination + 'static) {
        Self::with_router_mut(|router| {
            router.reset_with(root_dest);
        });
    }

    /// Number of destinations in the current scoped stack.
    pub fn len() -> usize {
        Self::with_router(ShardRouter::len)
    }

    /// Whether the current scoped stack is empty.
    pub fn is_empty() -> bool {
        Self::with_router(ShardRouter::is_empty)
    }
}

/// Provide a scoped router and execute child UI.
///
/// Nested router scopes are supported. Router commands always resolve to the
/// nearest scope.
pub fn router_scope<F>(root_dest: impl RouterDestination + 'static, content: F)
where
    F: FnOnce(),
{
    let scope_id = remember(RouterScopeId::new);
    let scope_id_value = scope_id.get();
    let router_state = remember(move || ShardRouter::with_root(scope_id_value, root_dest));
    provide_context(
        || RouterContext {
            state: router_state,
        },
        content,
    );
}

/// Render the current destination from the nearest scoped router.
///
/// # Panics
///
/// Panics if the scoped router stack is empty.
pub fn router_view() {
    let executed = Router::with_router(ShardRouter::exec_current);
    assert!(executed, "Router stack should not be empty");
}

/// Convenience helper equivalent to `router_scope(root_dest, router_view)`.
///
/// The root destination is only seeded when the scope state is first created.
/// It is not re-seeded automatically after explicit stack clears/pops.
pub fn router_root(root_dest: impl RouterDestination + 'static) {
    router_scope(root_dest, router_view);
}
