//! Image pipeline for vector (SVG) assets.

pub mod command;
pub mod pipeline;

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "image_vector/alpha_tests.rs"]
mod alpha_tests;
