use std::sync::Arc;

use tessera_ui::{
    __private::{
        RuntimePhase, context_from_previous_snapshot_for_instance, current_phase,
        current_replay_boundary_instance_key_from_scope,
    },
    State, provide_context, remember, tessera, use_context,
};

use crate::{
    router::{RouterContext, RouterController, RouterDestination},
    state::{ShardState, ShardStateLifeCycle},
};

fn resolve_router_controller_state() -> State<RouterController> {
    match current_phase() {
        Some(RuntimePhase::Build) => {
            let context = use_context::<RouterContext>()
                .expect("Router is missing in build scope. Mount UI inside shard_home.");
            context.get().controller
        }
        Some(RuntimePhase::Input) => {
            let instance_key = current_replay_boundary_instance_key_from_scope()
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

fn router_outlet() {
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
/// Mount the root route for an app shell.
///
/// ## Parameters
///
/// - `root` — initial destination used when `controller` is omitted
/// - `controller` — optional external router controller state
///
/// ## Examples
///
/// ```rust
/// use tessera_shard::shard_home;
///
/// # #[derive(Clone)]
/// # struct DemoDestination;
/// # impl tessera_shard::router::RouterDestination for DemoDestination {
/// #     fn exec_component(&self) {}
/// #     fn destination_id() -> &'static str { "demo" }
/// # }
/// # #[tessera_ui::tessera]
/// # fn demo() {
/// shard_home().root(DemoDestination);
/// # }
/// # demo();
/// ```
#[tessera(tessera_ui)]
pub fn shard_home(
    #[prop(skip_setter)] root: Option<Arc<dyn RouterDestination>>,
    controller: Option<State<RouterController>>,
) {
    let internal_controller = remember({
        let root = root.clone();
        move || match root.clone() {
            Some(root) => RouterController::with_root_shared(root),
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

    provide_context(|| RouterContext { controller }, router_outlet);
}

impl ShardHomeBuilder {
    pub fn root<T>(mut self, root: T) -> Self
    where
        T: RouterDestination + 'static,
    {
        self.props.root = Some(Arc::new(root));
        self
    }
}
