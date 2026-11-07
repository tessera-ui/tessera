//! GPU computation within UI rendering.
//!
//! ## Usage
//!
//! Submit [`ComputeCommand`] values to registered [`ComputablePipeline`] implementations.

pub mod command;
pub mod pipeline;

pub use command::ComputeCommand;
pub use pipeline::{
    ComputablePipeline, ComputeBatchItem, ComputePipelineRegistry, ErasedComputeBatchItem,
};
