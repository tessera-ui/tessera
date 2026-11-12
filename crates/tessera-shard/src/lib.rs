//! Navigation for Tessera applications.
//!
//! ## Usage
//!
//! Define route destinations whose view models live as long as their routes.

#![deny(
    missing_docs,
    clippy::unwrap_used,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::invalid_html_tags
)]

extern crate self as tessera_shard;

pub mod router;

pub use tessera_macros::shard;

pub use crate::router::{RouterController, RouterDestination, shard_home, use_router};
