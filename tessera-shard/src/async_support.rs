//! Async runtime integration and task orchestration.
//!
//! ## Usage
//!
//! Spawn and cancel shard-owned background tasks.

pub mod task_handles;

#[cfg(not(target_family = "wasm"))]
pub(crate) mod tokio_runtime;
