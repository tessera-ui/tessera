//! Mean luminance compute pipeline helpers.
//!
//! Provides the [`MeanCommand`] to request luminance sampling and the [`MeanPipeline`]
//! implementation that performs the GPU dispatch.

mod command;
mod pipeline;

pub use command::MeanCommand;
pub use pipeline::MeanPipeline;
