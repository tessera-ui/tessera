//! Blur pipeline module.
//!
//! This module provides GPU-based blur effect rendering pipelines and commands for use in Tessera UI components.
//!
//! # Example
//!
//! ```
//! use tessera_ui_basic_components::pipelines::blur::command::DualBlurCommand;
//! use tessera_ui_basic_components::pipelines::blur::pipeline::BlurPipeline;
//!
//! // Create and use BlurPipeline and DualBlurCommand in your rendering logic.
//! ```

/// Command definitions for dual-pass Gaussian blur.
pub mod command;
/// GPU pipeline implementation for blur rendering.
pub mod pipeline;
