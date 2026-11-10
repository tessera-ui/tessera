//! Shared asynchronous dispatch APIs for shard applications.
//!
//! ## Usage
//!
//! Use model-scoped tasks and return results to UI state through its dispatcher.

pub use tessera_ui::{Dispatcher, TaskHandle, dispatch};
