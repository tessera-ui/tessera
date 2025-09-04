//! Compute command trait and related types.
//!
//! This module defines the `ComputeCommand` trait that marks structs as compute operations
//! that can be processed by the GPU compute pipeline system.

use crate::{BarrierRequirement, dyn_eq_compute::DynPartialEqCompute, renderer::command::AsAny};

/// Trait for GPU compute operations that can be dispatched through the unified command system.
pub trait ComputeCommand: DynPartialEqCompute + AsAny + Send + Sync {
    fn barrier(&self) -> BarrierRequirement;
}
