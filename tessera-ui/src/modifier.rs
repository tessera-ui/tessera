//! Modifier chains for composing wrapper behavior around components.
//!
//! ## Usage
//!
//! Build reusable modifier chains for padding, backgrounds, and interactions.

use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use smallvec::SmallVec;

use crate::{prop::CallbackWith, runtime::ensure_build_phase};

/// A child subtree builder used by modifier wrappers.
pub type ModifierChild = Box<dyn Fn() + Send + Sync + 'static>;

/// A wrapper function that receives a child builder and returns a wrapped child
/// builder.
pub type ModifierWrapper = CallbackWith<ModifierChild, ModifierChild>;

#[derive(Clone)]
struct ModifierNode {
    prev: Option<Arc<ModifierNode>>,
    wrapper: ModifierWrapper,
}

fn collect_wrappers(mut node: Option<Arc<ModifierNode>>) -> Vec<ModifierWrapper> {
    let mut wrappers: SmallVec<[ModifierWrapper; 8]> = SmallVec::new();
    while let Some(current) = node {
        wrappers.push(current.wrapper.clone());
        node = current.prev.clone();
    }
    wrappers.into_vec()
}

/// A persistent handle to a modifier chain.
///
/// The modifier chain is immutable and replay-safe across frames.
#[derive(Clone, Default)]
pub struct Modifier {
    tail: Option<Arc<ModifierNode>>,
}

impl Modifier {
    /// Creates an empty modifier chain.
    pub fn new() -> Self {
        ensure_build_phase();
        Self::default()
    }

    /// Appends a wrapper action and returns the updated modifier.
    pub fn push_wrapper<F, Child>(self, wrapper: F) -> Self
    where
        F: Fn(ModifierChild) -> Child + Send + Sync + 'static,
        Child: Fn() + Send + Sync + 'static,
    {
        self.push_wrapper_shared(CallbackWith::new(
            move |child: ModifierChild| -> ModifierChild { Box::new(wrapper(child)) },
        ))
    }

    fn push_wrapper_shared(self, wrapper: ModifierWrapper) -> Self {
        ensure_build_phase();
        Self {
            tail: Some(Arc::new(ModifierNode {
                prev: self.tail,
                wrapper,
            })),
        }
    }

    /// Applies all wrappers to `child` and returns the wrapped child builder.
    pub fn apply<F>(self, child: F) -> ModifierChild
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.apply_child(Box::new(child))
    }

    fn apply_child(self, child: ModifierChild) -> ModifierChild {
        ensure_build_phase();

        if self.tail.is_none() {
            return child;
        }

        let wrappers = collect_wrappers(self.tail);
        wrappers
            .into_iter()
            .rev()
            .fold(child, |child, wrapper| wrapper.call(child))
    }

    /// Applies all wrappers to `child` and immediately runs the resulting tree.
    pub fn run<F>(self, child: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let child = self.apply(child);
        child();
    }
}

impl fmt::Debug for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Modifier")
            .field("is_empty", &self.tail.is_none())
            .finish()
    }
}

impl PartialEq for Modifier {
    fn eq(&self, other: &Self) -> bool {
        match (&self.tail, &other.tail) {
            (None, None) => true,
            (Some(lhs), Some(rhs)) => Arc::ptr_eq(lhs, rhs),
            _ => false,
        }
    }
}

impl Eq for Modifier {}

impl Hash for Modifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.tail {
            Some(node) => {
                // Pointer identity is the stable key for modifier replay/prop comparison.
                std::ptr::hash(Arc::as_ptr(node), state);
            }
            None => {
                0u8.hash(state);
            }
        }
    }
}
