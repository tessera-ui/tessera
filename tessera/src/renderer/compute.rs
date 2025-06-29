//! A generic system for offloading calculations to the GPU.
//!
//! This module provides a structured way to define, dispatch, and retrieve results from
//! GPU compute shaders. It is designed for asynchronous "fire-and-forget" operations
//! where the result is not needed immediately, allowing the GPU to work in parallel
//! without blocking the main render loop.
//!
//! # Key Components
//!
//! * [`ComputeCommand`]: A trait for structs that represent a request for a computation.
//! * [`ComputablePipeline`]: A trait for a specific compute task that processes a `ComputeCommand`.
//! * [`ComputePipelineRegistry`]: The central dispatcher that manages all registered pipelines.
//!
//! # Workflow
//!
//! 1.  **Define a Command:** Create a struct that holds all parameters for a computation.
//!     It must implement `ComputeCommand`, `Hash`, `Eq`, and `Clone`.
//! 2.  **Implement a Pipeline:** Create a struct that implements `ComputablePipeline<YourCommand>`.
//!     This involves writing the compute shader (WGSL), setting up the `wgpu::ComputePipeline`,
//!     and managing a cache for results.
//! 3.  **Register the Pipeline:** During application startup, register an instance of your
//!     pipeline with the `ComputePipelineRegistry`.
//! 4.  **Dispatch Commands:** From a rendering pipeline (e.g., `DrawablePipeline`), create a
//!     command and send it to the `ComputePipelineRegistry`.
//! 5.  **Retrieve Results:** On subsequent frames, attempt to retrieve the result from the
//!     registry. If it's not ready (`None`), the renderer can fall back to a simpler
//!     representation (e.g., a CPU-drawn shape). If it is ready (`Some`), the renderer
//!     can use the high-quality, GPU-generated resource.

pub mod command;
pub mod pipeline;

pub use command::ComputeCommand;
pub use pipeline::{ComputablePipeline, ComputePipelineRegistry};