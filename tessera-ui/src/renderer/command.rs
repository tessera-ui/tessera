//! Unified command system for rendering and computation.
//!
//! This module defines the `Command` enum that unifies draw and compute operations
//! into a single type, enabling seamless integration of graphics and compute pipelines
//! in the rendering workflow.

use std::any::Any;

use crate::{
    ComputeCommand, DrawCommand,
    px::{Px, PxRect},
};

/// Defines the sampling requirements for a rendering command that needs a barrier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BarrierRequirement {
    /// The command needs to sample from the entire previously rendered scene.
    /// This will cause a full-screen texture copy.
    Global,

    /// The command needs to sample from a region relative to its own bounding box.
    ///
    /// - Sampling padding: The region from which pixels are read (e.g., blur needs to read
    ///   pixels outside the target area). This determines the actual texture region captured.
    /// - Collision padding: The region used for overlap detection and batching decisions.
    ///   This can be smaller than sampling padding to allow better batching.
    ///
    /// For simple cases without special batching needs, set both paddings to the same value.
    /// For effects like blur that need large sampling areas but have small target areas,
    /// use large sampling padding but small (often zero) collision padding to allow batching
    /// of orthogonal components.
    ///
    /// # Examples
    ///
    /// Simple case (both paddings same):
    /// ```
    /// PaddedLocal {
    ///     sampling: { top: Px(10), right: Px(10), bottom: Px(10), left: Px(10) },
    ///     collision: { top: Px(10), right: Px(10), bottom: Px(10), left: Px(10) },
    /// }
    /// ```
    ///
    /// Blur optimization (large sampling, zero collision):
    /// ```
    /// PaddedLocal {
    ///     sampling: { top: Px(75), right: Px(75), bottom: Px(75), left: Px(75) },
    ///     collision: { top: Px(0), right: Px(0), bottom: Px(0), left: Px(0) },
    /// }
    /// ```
    PaddedLocal {
        sampling: PaddingRect,
        collision: PaddingRect,
    },

    /// The command needs to sample from a specific, absolute region of the screen.
    Absolute(PxRect),
}

/// Padding values for all four sides of a rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaddingRect {
    pub top: Px,
    pub right: Px,
    pub bottom: Px,
    pub left: Px,
}

impl PaddingRect {
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

impl BarrierRequirement {
    pub const ZERO_PADDING_LOCAL: Self = Self::PaddedLocal {
        sampling: PaddingRect::ZERO,
        collision: PaddingRect::ZERO,
    };

    /// Creates a `PaddedLocal` barrier requirement with uniform padding on all sides.
    /// Both sampling and collision use the same padding.
    #[must_use]
    pub const fn uniform_padding_local(padding: Px) -> Self {
        Self::PaddedLocal {
            sampling: PaddingRect::uniform(padding),
            collision: PaddingRect::uniform(padding),
        }
    }

    /// Creates a `PaddedLocal` barrier requirement with separate sampling and collision padding.
    ///
    /// Use this for effects that need a large sampling area (like blur) but where
    /// batching should be based on a smaller collision area (like the component itself).
    ///
    /// # Arguments
    ///
    /// * `sampling_padding` - Padding for the sampling region (e.g., blur radius * 1.5)
    /// * `collision_padding` - Padding for collision detection (often 0 for tight batching)
    #[must_use]
    pub const fn uniform_padding_local_with_collision(
        sampling_padding: Px,
        collision_padding: Px,
    ) -> Self {
        Self::PaddedLocal {
            sampling: PaddingRect::uniform(sampling_padding),
            collision: PaddingRect::uniform(collision_padding),
        }
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
    /// A graphics rendering command processed by draw pipelines
    Draw(Box<dyn DrawCommand>),
    /// A GPU computation command processed by compute pipelines
    Compute(Box<dyn ComputeCommand>),
    /// A command to push a clipping rectangle onto the stack
    ClipPush(PxRect),
    /// A command to pop the most recent clipping rectangle from the stack
    ClipPop,
}

impl Command {
    /// Returns the barrier requirement for this command.
    ///
    /// Commands that need to sample from previously rendered content
    /// should return a barrier requirement to ensure proper synchronization.
    #[must_use]
    pub fn barrier(&self) -> Option<BarrierRequirement> {
        match self {
            Self::Draw(command) => command.barrier(),
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

/// Automatic conversion from boxed draw commands to unified commands
impl From<Box<dyn DrawCommand>> for Command {
    fn from(val: Box<dyn DrawCommand>) -> Self {
        Self::Draw(val)
    }
}

/// Automatic conversion from boxed compute commands to unified commands
impl From<Box<dyn ComputeCommand>> for Command {
    fn from(val: Box<dyn ComputeCommand>) -> Self {
        Self::Compute(val)
    }
}
