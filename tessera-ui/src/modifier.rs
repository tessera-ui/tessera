//! Modifier chains for composing wrapper behavior around components.
//!
//! ## Usage
//!
//! Build reusable modifier chains for padding, backgrounds, and interactions.

use std::{
    cell::{Cell, RefCell},
    fmt,
    sync::Arc,
};

use smallvec::SmallVec;

use crate::runtime::ensure_build_phase;

/// A child subtree builder used by modifier wrappers.
pub type ModifierChild = Box<dyn FnOnce() + Send + Sync + 'static>;

/// A wrapper function that receives a child builder and returns a wrapped child
/// builder.
pub type ModifierWrapper = Arc<dyn Fn(ModifierChild) -> ModifierChild + Send + Sync + 'static>;

#[derive(Clone)]
struct ModifierNode {
    prev: u32,
    wrapper: ModifierWrapper,
}

thread_local! {
    static MODIFIER_ARENA: RefCell<Vec<ModifierNode>> = const { RefCell::new(Vec::new()) };
    static MODIFIER_GENERATION: Cell<u64> = const { Cell::new(0) };
}

fn current_generation() -> u64 {
    MODIFIER_GENERATION.with(|g| g.get())
}

fn ensure_fresh(modifier: Modifier) {
    let current = current_generation();
    if modifier.generation != current {
        panic!(
            "Modifier is stale (generation {}, current generation {})",
            modifier.generation, current
        );
    }
}

fn push_node(prev: u32, wrapper: ModifierWrapper) -> u32 {
    MODIFIER_ARENA.with(|arena| {
        let mut arena = arena.borrow_mut();
        let index = arena.len();
        let id = u32::try_from(index)
            .ok()
            .and_then(|v| v.checked_add(1))
            .expect("Modifier arena exhausted");
        arena.push(ModifierNode { prev, wrapper });
        id
    })
}

fn collect_wrappers(mut id: u32) -> Vec<ModifierWrapper> {
    MODIFIER_ARENA.with(|arena| {
        let arena = arena.borrow();
        let mut wrappers: SmallVec<[ModifierWrapper; 8]> = SmallVec::new();
        while id != 0 {
            let idx = (id - 1) as usize;
            let node = arena
                .get(idx)
                .unwrap_or_else(|| panic!("Invalid Modifier id {}", id));
            wrappers.push(node.wrapper.clone());
            id = node.prev;
        }
        wrappers.into_vec()
    })
}

/// A `Copy` handle to a modifier chain.
///
/// This handle is only valid for the current build phase. Using it after the
/// frame ends will panic.
#[derive(Clone, Copy)]
pub struct Modifier {
    id: u32,
    generation: u64,
}

impl Modifier {
    /// Creates an empty modifier chain.
    pub fn new() -> Self {
        ensure_build_phase();
        Self {
            id: 0,
            generation: current_generation(),
        }
    }

    /// Appends a wrapper action and returns the updated modifier.
    pub fn push_wrapper<F, Child>(self, wrapper: F) -> Self
    where
        F: Fn(ModifierChild) -> Child + Send + Sync + 'static,
        Child: FnOnce() + Send + Sync + 'static,
    {
        self.push_wrapper_shared(Arc::new(move |child| Box::new(wrapper(child))))
    }

    fn push_wrapper_shared(self, wrapper: ModifierWrapper) -> Self {
        ensure_build_phase();
        ensure_fresh(self);
        let id = push_node(self.id, wrapper);
        Self { id, ..self }
    }

    /// Applies all wrappers to `child` and returns the wrapped child builder.
    pub fn apply<F>(self, child: F) -> ModifierChild
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.apply_child(Box::new(child))
    }

    fn apply_child(self, child: ModifierChild) -> ModifierChild {
        ensure_build_phase();
        ensure_fresh(self);

        if self.id == 0 {
            return child;
        }

        let wrappers = collect_wrappers(self.id);
        wrappers
            .into_iter()
            .rev()
            .fold(child, |child, wrapper| (wrapper)(child))
    }

    /// Applies all wrappers to `child` and immediately runs the resulting tree.
    pub fn run<F>(self, child: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        let child = self.apply(child);
        child();
    }
}

impl Default for Modifier {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Modifier")
            .field("id", &self.id)
            .field("generation", &self.generation)
            .finish()
    }
}

pub(crate) fn clear_modifiers() {
    MODIFIER_ARENA.with(|arena| arena.borrow_mut().clear());
    MODIFIER_GENERATION.with(|generation| generation.set(generation.get().wrapping_add(1)));
}
