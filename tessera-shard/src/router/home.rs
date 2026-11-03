use std::sync::Arc;

use tessera_ui::{State, remember, tessera};

use crate::{
    router::{RouterController, RouterDestination},
    state::{ShardState, ShardStateLifeCycle},
};

pub(crate) fn with_current_router_shard_state<T, F, R>(
    shard_id: &str,
    life_cycle: ShardStateLifeCycle,
    controller: State<RouterController>,
    f: F,
) -> R
where
    T: Default + Send + Sync + 'static,
    F: FnOnce(ShardState<T>) -> R,
{
    let (owner, marker) = controller.with(|router| router.state_context(life_cycle));
    let value = owner.state_with_key(("shard", shard_id), T::default);
    f(ShardState::new(value, marker))
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
/// #     fn exec_component(&self, _: tessera_ui::State<tessera_shard::RouterController>) {}
/// #     fn exec_component_handle(&self) -> std::sync::Arc<dyn Fn(tessera_ui::State<tessera_shard::RouterController>) + Send + Sync> {
/// #         std::sync::Arc::new(|_| {})
/// #     }
/// #     fn destination_id() -> &'static str { "demo" }
/// # }
/// # #[tessera_ui::tessera]
/// # fn demo() {
/// shard_home().root(DemoDestination);
/// # }
/// # tessera_ui::testing::layout_test(demo).run();
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

    let destination = controller
        .with(RouterController::current_destination)
        .expect("Router stack should not be empty");
    destination(controller);
}

impl ShardHomeBuilder {
    pub fn root<T>(&self, root: T) -> Self
    where
        T: RouterDestination + 'static,
    {
        let value: Option<Arc<dyn RouterDestination>> = Some(Arc::new(root));
        self.props
            .configure("root", move |props| props.root = value);
        Self { props: self.props }
    }
}
