use std::sync::atomic::{AtomicU64, Ordering};

use tessera_ui::State;

use crate::router::RouterController;

static NEXT_ROUTE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub(crate) struct RouterContext {
    pub(crate) controller: State<RouterController>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct RouteId(pub(crate) u64);

impl RouteId {
    pub(crate) fn new() -> Self {
        Self(NEXT_ROUTE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct RouteShardKey {
    pub(crate) route_id: RouteId,
    pub(crate) shard_id: String,
}
