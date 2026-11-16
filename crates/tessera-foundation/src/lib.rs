//! Foundation primitives and shared UI building blocks for Tessera.
//!
//! ## Usage
//!
//! Build reusable UI with layout, shape, and interaction primitives.
//! Import [`modifier::ModifierExt`] for clickable, combined-clickable, toggleable,
//! and selectable behavior with keyboard, focus, and accessibility support.
//! Use [`pointer_gestures`] for custom gesture recognition without those behaviors.
#![deny(
    missing_docs,
    clippy::unwrap_used,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::invalid_html_tags
)]

pub mod alignment;
pub mod modifier;
pub mod pointer_gestures;
pub mod scroll_physics;
pub mod shape_def;
