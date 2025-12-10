//! This module provides a workaround for dynamic equality testing of trait
//! objects of the `ComputeCommand` trait.

use std::any::Any;

use crate::ComputeCommand;

/// A trait that allows for cloning a trait object.
pub trait DynCloneCompute {
    /// Creates a boxed clone of the trait object.
    fn clone_box(&self) -> Box<dyn ComputeCommand>;
}

impl<T> DynCloneCompute for T
where
    T: ComputeCommand + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn ComputeCommand> {
        Box::new(self.clone())
    }
}

/// A trait that allows for dynamic equality testing of trait objects.
///
/// This trait provides a workaround for the fact that `PartialEq` is not
/// object-safe. It allows types that are `PartialEq` to be compared even when
/// they are behind a trait object by downcasting them to their concrete types.
pub trait DynPartialEqCompute: DynCloneCompute {
    /// Returns the object as a `&dyn Any`.
    fn as_any(&self) -> &dyn Any;

    /// Performs a dynamic equality check against another `DynPartialEqCompute`
    /// trait object.
    fn dyn_eq(&self, other: &dyn DynPartialEqCompute) -> bool;
}

impl<T: ComputeCommand + PartialEq + 'static> DynPartialEqCompute for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn DynPartialEqCompute) -> bool {
        // Attempt to downcast the `other` trait object to the same concrete type as
        // `self`.
        if let Some(other_concrete) = other.as_any().downcast_ref::<T>() {
            // If the downcast is successful, perform the actual comparison.
            self == other_concrete
        } else {
            // If the types are different, they cannot be equal.
            false
        }
    }
}
