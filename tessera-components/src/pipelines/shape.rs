//! Shape pipeline for filled, outlined, and ripple-animated primitives.

pub mod command;
pub mod pipeline;

#[cfg(all(test, not(target_family = "wasm")))]
mod coverage_tests;
