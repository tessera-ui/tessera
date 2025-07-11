//! A unified system for GPU-based computation.
//!
//! This module provides a structured way to define and dispatch compute shaders as part
//! of a sequential rendering and computation workflow. It integrates seamlessly with the
//! unified command system to enable mixed graphics and compute workloads.
//!
//! # Key Components
//!
//! * [`ComputeCommand`]: A trait marking a command as a compute operation with optional barrier support.
//! * [`ComputablePipeline`]: A trait for a specific compute task that processes a command
//!   within a `wgpu::ComputePass`.
//! * [`ComputePipelineRegistry`]: The central dispatcher that manages all registered pipelines.
//!
//! # Workflow
//!
//! 1.  **Define a Command:** Create a struct that implements `ComputeCommand`.
//! 2.  **Implement a Pipeline:** Create a struct that implements `ComputablePipeline<YourCommand>`.
//!     This involves setting up the `wgpu::ComputePipeline` and defining the `dispatch` method.
//! 3.  **Register the Pipeline:** During application startup, register an instance of your
//!     pipeline with the `ComputePipelineRegistry`.
//! 4.  **Submit Commands:** Components submit `ComputeCommand`s, which are then dispatched
//!     by the renderer to the appropriate pipeline through the unified command system.
//!
//! # Barrier Support
//!
//! Compute commands can specify barrier requirements to ensure proper synchronization
//! with previous rendering operations, enabling post-processing effects and multi-pass algorithms.

pub mod command;
pub mod pipeline;

pub use command::ComputeCommand;
pub use pipeline::{ComputablePipeline, ComputePipelineRegistry};
