//! Async runtime integration for shard-owned background tasks.
//!
//! ## Usage
//!
//! Group background jobs in `TaskHandles` and cancel them with shard teardown.

/// Task handle types for spawning and canceling shard-scoped jobs.
pub mod task_handles;

#[cfg(not(target_family = "wasm"))]
pub(crate) mod tokio_runtime;
