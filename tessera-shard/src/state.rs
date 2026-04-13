//! Shard state handle and storage primitives.
//!
//! ## Usage
//!
//! Use `ShardState` to access shard-scoped data managed by the router.

pub mod handle;
pub mod storage;

pub use handle::ShardState;

pub(crate) use storage::{
    ShardStateMap, ShardStateSlot, init_or_get_shard_state_in_map, recycle_shard_state_slot,
};

/// Describes the lifecycle of shard state values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ShardStateLifeCycle {
    /// State exists for the lifetime of one router controller instance.
    Scope,
    /// State exists for the lifetime of a route instance.
    Shard,
}
