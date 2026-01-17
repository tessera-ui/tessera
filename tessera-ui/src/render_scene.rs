//! Render command definitions for frame graphs.
//!
//! ## Usage
//!
//! Define command metadata for render graph nodes.

use std::any::Any;

use crate::{
    ComputeCommand, DrawCommand,
    px::{Px, PxRect},
};

/// Defines the sampling requirements for a rendering command that needs a
/// barrier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SampleRegion {
    /// The command needs to sample from the entire previously rendered scene.
    /// This will cause a full-screen texture copy.
    Global,
    /// The command needs to sample from a region relative to its own bounding
    /// box.
    PaddedLocal(PaddingRect),
    /// The command needs to sample from a specific, absolute region of the
    /// screen.
    Absolute(PxRect),
}

/// Defines the drawing region for a rendering command.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawRegion {
    /// The command draws to the entire surface.
    Global,
    /// The command draws to a region relative to its own bounding box.
    PaddedLocal(PaddingRect),
    /// The command draws to a specific, absolute region of the screen.
    Absolute(PxRect),
}

/// Padding values for all four sides of a rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaddingRect {
    /// Padding applied to the top edge.
    pub top: Px,
    /// Padding applied to the right edge.
    pub right: Px,
    /// Padding applied to the bottom edge.
    pub bottom: Px,
    /// Padding applied to the left edge.
    pub left: Px,
}

impl PaddingRect {
    /// A zero padding rectangle that leaves the sampling region unchanged.
    pub const ZERO: Self = Self {
        top: Px::ZERO,
        right: Px::ZERO,
        bottom: Px::ZERO,
        left: Px::ZERO,
    };

    /// Creates a uniform padding rectangle with the same padding on all sides.
    #[must_use]
    pub const fn uniform(padding: Px) -> Self {
        Self {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }
}

impl SampleRegion {
    /// A zero-padding local barrier requirement for commands that only sample
    /// within their bounds.
    pub const ZERO_PADDING_LOCAL: Self = Self::PaddedLocal(PaddingRect::ZERO);

    /// Creates a `PaddedLocal` barrier requirement with uniform sampling
    /// padding on all sides.
    #[must_use]
    pub const fn uniform_padding_local(padding: Px) -> Self {
        Self::PaddedLocal(PaddingRect::uniform(padding))
    }
}

/// Trait providing type erasure capabilities for command objects.
///
/// This trait allows commands to be stored and passed around as trait objects
/// while still providing access to their concrete types when needed for
/// pipeline dispatch.
pub trait AsAny {
    /// Returns a reference to the concrete type as `&dyn Any`.
    fn as_any(&self) -> &dyn Any;
}

/// Blanket implementation of `AsAny` for all types that implement `Any`.
impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Unified command enum that can represent either a draw or compute operation.
///
/// This enum enables the rendering system to process both graphics and compute
/// commands in a unified pipeline, with proper barrier handling for multi-pass
/// rendering scenarios.
pub enum Command {
    /// A graphics rendering command processed by draw pipelines.
    Draw(Box<dyn DrawCommand>),
    /// A GPU computation command processed by compute pipelines.
    Compute(Box<dyn ComputeCommand>),
    /// A command to push a clipping rectangle onto the stack.
    ClipPush(PxRect),
    /// A command to pop the most recent clipping rectangle from the stack.
    ClipPop,
}

impl Command {
    /// Returns the barrier requirement for this command.
    ///
    /// Commands that need to sample from previously rendered content
    /// should return a barrier requirement to ensure proper synchronization.
    #[must_use]
    pub fn barrier(&self) -> Option<SampleRegion> {
        match self {
            Self::Draw(command) => command.sample_region(),
            // Currently, compute can only be used for after effects,
            Self::Compute(command) => Some(command.barrier()),
            Self::ClipPush(_) | Self::ClipPop => None, // Clipping commands do not require barriers
        }
    }
}

impl Clone for Command {
    fn clone(&self) -> Self {
        match self {
            Self::Draw(cmd) => Self::Draw(cmd.clone_box()),
            Self::Compute(cmd) => Self::Compute(cmd.clone_box()),
            Self::ClipPush(rect) => Self::ClipPush(*rect),
            Self::ClipPop => Self::ClipPop,
        }
    }
}

/// Automatic conversion from boxed draw commands to unified commands.
impl From<Box<dyn DrawCommand>> for Command {
    fn from(val: Box<dyn DrawCommand>) -> Self {
        Self::Draw(val)
    }
}

/// Automatic conversion from boxed compute commands to unified commands.
impl From<Box<dyn ComputeCommand>> for Command {
    fn from(val: Box<dyn ComputeCommand>) -> Self {
        Self::Compute(val)
    }
}
