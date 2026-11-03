use std::hash::{Hash, Hasher};

use tessera_ui::State;

/// Failure while accessing state after its owning UI lifetime ends.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShardStateError {
    /// The controller or its UI session has been released.
    ControllerDropped,
    /// The route that owned this value has left the stack.
    Stale,
}

/// Reactive state owned by a route or router controller.
///
/// Reads subscribe the current UI phase to changes. Handles remain valid while
/// their owner lives and must be accessed on the owning UI session.
pub struct ShardState<T> {
    value: State<T>,
    controller: State<()>,
}

impl<T> Copy for ShardState<T> {}
impl<T> Clone for ShardState<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> PartialEq for ShardState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.controller == other.controller
    }
}
impl<T> Eq for ShardState<T> {}
impl<T> Hash for ShardState<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.controller.hash(state);
    }
}

impl<T: Send + Sync + 'static> ShardState<T> {
    pub(crate) fn new(value: State<T>, controller: State<()>) -> Self {
        Self { value, controller }
    }

    fn check(&self) -> Result<(), ShardStateError> {
        if !self.controller.is_alive() {
            Err(ShardStateError::ControllerDropped)
        } else if !self.value.is_alive() {
            Err(ShardStateError::Stale)
        } else {
            Ok(())
        }
    }

    /// Reads the value and subscribes the active UI phase to its changes.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.try_with(f)
            .unwrap_or_else(|error| panic!("ShardState access failed: {error:?}"))
    }

    /// Updates the value and invalidates its UI consumers.
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.try_with_mut(f)
            .unwrap_or_else(|error| panic!("ShardState access failed: {error:?}"))
    }

    /// Reads live state, reporting an expired controller or route.
    pub fn try_with<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, ShardStateError> {
        self.check()?;
        Ok(self.value.with(f))
    }

    /// Updates live state, reporting an expired controller or route.
    pub fn try_with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, ShardStateError> {
        self.check()?;
        Ok(self.value.with_mut(f))
    }

    /// Returns a cloned value.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with(Clone::clone)
    }

    /// Replaces the value and invalidates its UI consumers.
    pub fn set(&self, value: T) {
        self.with_mut(|current| *current = value);
    }
}
