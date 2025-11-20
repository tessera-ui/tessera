//! Checkmark pipeline module.
//!
//! This module provides rendering pipelines and commands for drawing checkmark graphics in Tessera UI components.
//!
//! # Example
//!
//! ```
//! use tessera_ui_basic_components::pipelines::checkmark::CheckmarkCommand;
//! use tessera_ui_basic_components::pipelines::checkmark::CheckmarkPipeline;
//! // Use CheckmarkPipeline and CheckmarkCommand in your rendering logic.
//! ```

mod command;
mod pipeline;

/// Command for rendering a checkmark shape in UI components.
pub use command::CheckmarkCommand;

/// Pipeline for rendering checkmark graphics.
pub use pipeline::CheckmarkPipeline;

