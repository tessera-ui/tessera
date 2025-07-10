//! A generic system for offloading calculations to the GPU.
//!
//! This module provides a structured way to define and dispatch synchronous GPU compute shaders.
//!
//! # Key Components
//!
//! * [`SyncComputablePipeline`]: A trait for a specific compute task that processes a command.
//! * [`ComputePipelineRegistry`]: The central dispatcher that manages all registered pipelines.
//!
//! # Workflow
//!
//! 1.  **Define a Command:** Create a struct or tuple that holds all parameters for a computation.
//!     This will be the `Command` associated type in the pipeline.
//! 2.  **Implement a Pipeline:** Create a struct that implements `SyncComputablePipeline`.
//!     This involves writing the compute shader (WGSL), setting up the `wgpu::ComputePipeline`,
//!     and defining the `dispatch_sync` method.
//! 3.  **Register the Pipeline:** During application startup, register an instance of your
//!     pipeline with the `ComputePipelineRegistry`.
//! 4.  **Dispatch Commands:** Retrieve the pipeline from the registry using `get_sync` and call
//!     `dispatch_sync` directly.

pub mod command;
pub mod pipeline;

pub use pipeline::{ComputePipelineRegistry, SyncComputablePipeline};
