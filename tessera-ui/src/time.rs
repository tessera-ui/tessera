//! Cross-platform monotonic time helpers.
//!
//! ## Usage
//!
//! Use `Instant` for frame timing and animations on both native and web
//! targets.

#[cfg(not(target_family = "wasm"))]
pub use std::time::Instant;

#[cfg(target_family = "wasm")]
pub use web_time::Instant;
