//! Contrast pipeline module.
//!
//! This module exposes the [`ContrastCommand`] and [`ContrastPipeline`] types, which together
//! implement a compute-driven contrast adjustment pass for Tessera UI components.

mod command;
mod pipeline;

pub use command::ContrastCommand;
pub use pipeline::ContrastPipeline;
