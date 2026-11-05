//! Shard state handle and storage primitives.
//!
//! ## Usage
//!
//! Use `ShardState` to access shard-scoped data managed by the router.

/// Public handle APIs for shard-scoped state access.
pub mod handle;

pub use handle::{ShardState, ShardStateError};

/// Describes the lifecycle of shard state values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ShardStateLifeCycle {
    /// State exists for the lifetime of one router controller instance.
    Scope,
    /// State exists for the lifetime of a route instance.
    Shard,
}
