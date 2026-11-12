//! GPU computation within UI rendering.
//!
//! ## Usage
//!
//! Submit [`ComputeCommand`] values to registered [`ComputablePipeline`] implementations.

mod command;
mod pipeline;

pub use command::ComputeCommand;
pub use pipeline::{ComputablePipeline, ComputeBatchItem, ComputeContext, ComputePipelineRegistry};

pub(crate) use pipeline::{ErasedComputeBatchItem, ErasedDispatchContext};
