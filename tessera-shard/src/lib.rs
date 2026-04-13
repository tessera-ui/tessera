//! Shard routing, state, and async support for Tessera applications.
//!
//! ## Usage
//!
//! Define route destinations and shard-scoped state for router-driven screens.

#![deny(
    missing_docs,
    clippy::unwrap_used,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::invalid_html_tags
)]

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
