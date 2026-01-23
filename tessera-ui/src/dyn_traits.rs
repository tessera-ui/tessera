//! Dynamic trait helpers for command objects.
//!
//! ## Usage
//!
//! Enable dynamic equality checks for cloneable command trait objects.

use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::DynClone;

use crate::{ComputeCommand, renderer::drawer::DrawCommand};

macro_rules! define_dyn_partial_eq {
    ($trait_name:ident, $bound:path) => {
        /// A trait that allows dynamic equality testing for trait objects.
        pub trait $trait_name: DynClone + Downcast {
            /// Performs a dynamic equality check against another trait object.
            fn dyn_eq(&self, other: &dyn $trait_name) -> bool;
        }

        impl<T> $trait_name for T
        where
            T: $bound + PartialEq + 'static,
        {
            fn dyn_eq(&self, other: &dyn $trait_name) -> bool {
                other
                    .downcast_ref::<T>()
                    .is_some_and(|other_concrete| self == other_concrete)
            }
        }
    };
}

define_dyn_partial_eq!(DynPartialEqDraw, DrawCommand);
define_dyn_partial_eq!(DynPartialEqCompute, ComputeCommand);

impl_downcast!(DynPartialEqDraw);
impl_downcast!(DynPartialEqCompute);
