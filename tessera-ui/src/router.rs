//! Controller-driven shard routing primitives.
//!
//! ## Usage
//!
//! Mount `shard_home` at the app shell root, then render `router_outlet` where
//! the current shard page should appear.

use std::sync::Arc;

use tessera_macros::tessera;

pub use tessera_shard::{
    ShardState, ShardStateLifeCycle,
    router::{RouterController, RouterDestination},
};

use crate::{
    RenderSlot, State,
    context::{context_from_previous_snapshot_for_instance, provide_context, use_context},
    runtime::{RuntimePhase, current_component_instance_key_from_scope, current_phase, remember},
};

#[derive(Clone)]
struct RouterContext {
    controller: State<RouterController>,
}

/// Shared destination handle used by `RouterController` builders and
/// `shard_home`.
pub struct RouterDestinationHandle {
    inner: Arc<dyn RouterDestination>,
}

impl RouterDestinationHandle {
    fn clone_destination(&self) -> Arc<dyn RouterDestination> {
        Arc::clone(&self.inner)
    }
}

impl<T> From<T> for RouterDestinationHandle
where
    T: RouterDestination + 'static,
{
    fn from(value: T) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }
}

impl Clone for RouterDestinationHandle {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl PartialEq for RouterDestinationHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for RouterDestinationHandle {}

fn resolve_router_controller_state() -> State<RouterController> {
    match current_phase() {
        Some(RuntimePhase::Build) => {
            let context = use_context::<RouterContext>()
                .expect("Router is missing in build scope. Mount UI inside shard_home.");
            context.get().controller
        }
        Some(RuntimePhase::Input) => {
            let instance_key = current_component_instance_key_from_scope()
                .expect("Router command requires an active component scope during input handling");
            let context = context_from_previous_snapshot_for_instance::<RouterContext>(
                instance_key,
            )
            .expect("Router is missing in input scope. Ensure callbacks run inside shard_home.");
            context.get().controller
        }
        _ => {
            panic!("Router access must happen during build or input phase");
        }
    }
}

pub(crate) fn current_router_controller() -> State<RouterController> {
    resolve_router_controller_state()
}

pub(crate) fn with_current_router_shard_state<T, F, R>(
    shard_id: &str,
    life_cycle: ShardStateLifeCycle,
    f: F,
) -> R
where
    T: Default + Send + Sync + 'static,
    F: FnOnce(ShardState<T>) -> R,
{
    let controller = current_router_controller();
    controller.with(|router| router.init_or_get_with_lifecycle(shard_id, life_cycle, f))
}

/// Render the current destination from the nearest shard home.
pub fn router_outlet() {
    let executed = current_router_controller().with(RouterController::exec_current);
    assert!(executed, "Router stack should not be empty");
}

/// # shard_home
///
/// Provide a router controller and render shard UI rooted at the active
/// destination.
///
/// ## Usage
///
/// Mount the root route for an app shell and optionally render custom chrome
/// around `router_outlet()`.
///
/// ## Parameters
///
/// - `root` — initial destination used when `controller` is omitted
/// - `controller` — optional external router controller state
/// - `child` — optional shell content; defaults to `router_outlet()`
///
/// ## Examples
///
/// ```rust
/// use tessera_ui::router::shard_home;
///
/// # #[derive(Clone)]
/// # struct DemoDestination;
/// # impl tessera_ui::router::RouterDestination for DemoDestination {
/// #     fn exec_component(&self) {}
/// #     fn shard_id(&self) -> &'static str { "demo" }
/// # }
/// # #[tessera_ui::tessera]
/// # fn demo() {
/// shard_home().root(DemoDestination);
/// # }
/// # demo();
/// ```
#[tessera(crate)]
pub fn shard_home(
    #[prop(into)] root: Option<RouterDestinationHandle>,
    controller: Option<State<RouterController>>,
    child: Option<RenderSlot>,
) {
    let internal_controller = remember({
        let root = root.clone();
        move || match root.clone() {
            Some(root) => RouterController::with_root_shared(root.clone_destination()),
            None => RouterController::new(),
        }
    });
    let controller = controller.unwrap_or(internal_controller);

    if root.is_none()
        && controller == internal_controller
        && controller.with(RouterController::is_empty)
    {
        panic!("shard_home requires `root` when `controller` is not provided");
    }

    provide_context(
        || RouterContext { controller },
        move || {
            if let Some(child) = child.clone() {
                child.render();
            } else {
                router_outlet();
            }
        },
    );
}
