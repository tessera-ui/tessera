//! Cross-platform clocks for elapsed time and calendar dates.
//!
//! ## Usage
//!
//! Use `Instant` for animations and `SystemTime` for calendar features.

#[cfg(not(target_family = "wasm"))]
pub use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[cfg(target_family = "wasm")]
pub use web_time::{Instant, SystemTime, UNIX_EPOCH};
