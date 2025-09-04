use std::any::Any;

use crate::renderer::drawer::DrawCommand;

/// A trait that allows for cloning a trait object.
pub trait DynCloneDraw {
    /// Creates a boxed clone of the trait object.
    fn clone_box(&self) -> Box<dyn DrawCommand>;
}

impl<T> DynCloneDraw for T
where
    T: DrawCommand + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn DrawCommand> {
        Box::new(self.clone())
    }
}

/// A trait that allows for dynamic equality testing of trait objects.
///
/// This trait provides a workaround for the fact that `PartialEq` is not object-safe.
/// It allows types that are `PartialEq` to be compared even when they are behind
/// a trait object by downcasting them to their concrete types.
pub trait DynPartialEqDraw: DynCloneDraw {
    /// Returns the object as a `&dyn Any`.
    fn as_any(&self) -> &dyn Any;

    /// Performs a dynamic equality check against another `DynPartialEqDraw` trait object.
    fn dyn_eq(&self, other: &dyn DynPartialEqDraw) -> bool;
}

impl<T: DrawCommand + PartialEq + 'static> DynPartialEqDraw for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn DynPartialEqDraw) -> bool {
        // Attempt to downcast the `other` trait object to the same concrete type as `self`.
        if let Some(other_concrete) = other.as_any().downcast_ref::<T>() {
            // If the downcast is successful, perform the actual comparison.
            self == other_concrete
        } else {
            // If the types are different, they cannot be equal.
            false
        }
    }
}
