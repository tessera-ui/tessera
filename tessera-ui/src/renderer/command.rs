//! Unified command system for rendering and computation.
//!
//! This module defines the `Command` enum that unifies draw and compute operations
//! into a single type, enabling seamless integration of graphics and compute pipelines
//! in the rendering workflow.

use crate::{BarrierRequirement, ComputeCommand, DrawCommand};

/// Unified command enum that can represent either a draw or compute operation.
///
/// This enum enables the rendering system to process both graphics and compute
/// commands in a unified pipeline, with proper barrier handling for multi-pass
/// rendering scenarios.
///
/// # Examples
///
/// ```rust,ignore
/// // Creating a draw command
/// let draw_cmd = Command::Draw(Box::new(ShapeCommand::Rect { /* ... */ }));
///
/// // Creating a compute command
/// let compute_cmd = Command::Compute(Box::new(BlurCommand { /* ... */ }));
/// ```
pub enum Command {
    /// A graphics rendering command processed by draw pipelines
    Draw(Box<dyn DrawCommand>),
    /// A GPU computation command processed by compute pipelines
    Compute(Box<dyn ComputeCommand>),
}

impl Command {
    /// Returns the barrier requirement for this command.
    ///
    /// Commands that need to sample from previously rendered content
    /// should return a barrier requirement to ensure proper synchronization.
    pub fn barrier(&self) -> Option<BarrierRequirement> {
        match self {
            Command::Draw(command) => command.barrier(),
            // Currently, compute can only be used for after effects,
            // so we assume it must require a barrier to sample background.
            Command::Compute(_) => Some(BarrierRequirement::SampleBackground),
        }
    }
}

/// Automatic conversion from boxed draw commands to unified commands
impl From<Box<dyn DrawCommand>> for Command {
    fn from(val: Box<dyn DrawCommand>) -> Self {
        Command::Draw(val)
    }
}

/// Automatic conversion from boxed compute commands to unified commands
impl From<Box<dyn ComputeCommand>> for Command {
    fn from(val: Box<dyn ComputeCommand>) -> Self {
        Command::Compute(val)
    }
}
