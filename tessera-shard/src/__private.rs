#![allow(missing_docs)]

use tessera_ui::State;

use crate::{ShardState, ShardStateLifeCycle, router::RouterController};

pub fn current_router_controller() -> State<RouterController> {
    crate::router::current_router_controller()
}

pub fn with_current_router_shard_state<T, F, R>(
    shard_id: &str,
    life_cycle: ShardStateLifeCycle,
    f: F,
) -> R
where
    T: Default + Send + Sync + 'static,
    F: FnOnce(ShardState<T>) -> R,
{
    crate::router::with_current_router_shard_state(shard_id, life_cycle, f)
}
