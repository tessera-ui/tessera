//! Compute command trait and related types.
//!
//! This module defines the `ComputeCommand` trait that marks structs as compute
//! operations that can be processed by the GPU compute pipeline system.

use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::DynClone;

use crate::SampleRegion;

/// Trait for GPU compute operations that can be dispatched through the unified
/// command system.
pub trait ComputeCommand: DynClone + Downcast + Send + Sync {
    /// Declares the dependency on previously rendered content for barrier
    /// planning.
    fn barrier(&self) -> SampleRegion;
}

impl_downcast!(ComputeCommand);

dyn_clone::clone_trait_object!(ComputeCommand);
