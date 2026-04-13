#[doc(hidden)]
pub mod __private;
pub mod async_support;
pub mod router;
pub mod state;

pub use tessera_macros::shard;

pub use crate::{
    async_support::task_handles,
    router::{RouterController, RouterDestination, shard_home},
    state::{ShardState, ShardStateLifeCycle},
};
