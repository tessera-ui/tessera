#![allow(missing_docs)]

use tessera_ui::State;

use crate::{ShardState, ShardStateLifeCycle, router::RouterController};

#[derive(Clone, Copy)]
pub struct RouteRenderScope {
    pub(crate) controller: State<RouterController>,
    pub(crate) route_id: u64,
}

pub fn with_rendered_router_shard_state<T, F, R>(
    shard_id: &str,
    life_cycle: ShardStateLifeCycle,
    controller: State<RouterController>,
    scope: Option<RouteRenderScope>,
    f: F,
) -> R
where
    T: Default + Send + Sync + 'static,
    F: FnOnce(ShardState<T>) -> R,
{
    let route = scope
        .filter(|scope| scope.controller == controller)
        .map(|scope| scope.route_id);
    let (model, marker) = controller.with(|router| router.route_state_context(life_cycle, route));
    let value = model.state_with_key(("shard", shard_id), T::default);
    f(ShardState::new(value, marker))
}

pub fn with_current_router_shard_state<T, F, R>(
    shard_id: &str,
    life_cycle: ShardStateLifeCycle,
    controller: State<RouterController>,
    f: F,
) -> R
where
    T: Default + Send + Sync + 'static,
    F: FnOnce(ShardState<T>) -> R,
{
    with_rendered_router_shard_state(shard_id, life_cycle, controller, None, f)
}
