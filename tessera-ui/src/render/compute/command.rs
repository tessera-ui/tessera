//! Compute command trait and related types.

use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::DynClone;

use crate::render::SampleRegion;

/// Trait for GPU compute operations that can be dispatched through the unified
/// command system.
pub trait ComputeCommand: DynClone + Downcast + Send + Sync {
    /// Scene region this command reads.
    fn barrier(&self) -> SampleRegion;
}

impl_downcast!(ComputeCommand);

dyn_clone::clone_trait_object!(ComputeCommand);
