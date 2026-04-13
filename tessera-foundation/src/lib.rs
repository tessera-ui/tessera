//! Foundation primitives and shared UI building blocks for Tessera.
//!
//! ## Usage
//!
//! Use foundational types such as alignment and shape definitions when building
//! reusable layout and visual APIs.
#![deny(
    missing_docs,
    clippy::unwrap_used,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::invalid_html_tags
)]

pub mod alignment;
pub mod gesture;
pub mod modifier;
pub mod shape_def;
