//! Explicit inputs and observations for UI test fixtures.
//!
//! ## Usage
//!
//! Pass probes through component props and capture them in event callbacks.

use std::sync::{Arc, Mutex};

/// An observation or input owned by one test fixture.
///
/// Pass a clone through component props or capture it in callbacks. Clones refer
/// to the same fixture value; independently created probes remain independent.
pub struct TestProbe<T>(Arc<Mutex<T>>);

impl<T> Clone for TestProbe<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> PartialEq for TestProbe<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for TestProbe<T> {}

impl<T: Default> Default for TestProbe<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> TestProbe<T> {
    /// Creates a value for one fixture.
    pub fn new(value: T) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }

    /// Reads the current observation.
    pub fn with<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        read(&self.0.lock().unwrap_or_else(|error| error.into_inner()))
    }

    /// Updates the fixture value without creating UI dependencies.
    pub fn with_mut<R>(&self, update: impl FnOnce(&mut T) -> R) -> R {
        update(&mut self.0.lock().unwrap_or_else(|error| error.into_inner()))
    }

    /// Replaces the fixture value.
    pub fn set(&self, value: T) {
        let previous = self.with_mut(|slot| std::mem::replace(slot, value));
        drop(previous);
    }

    /// Copies the current fixture value.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with(Clone::clone)
    }

    /// Takes the fixture value, leaving its default value.
    pub fn take(&self) -> T
    where
        T: Default,
    {
        self.with_mut(std::mem::take)
    }
}

impl<T: Send + 'static> TestProbe<Vec<T>> {
    /// Creates an owned callback that records each observed event.
    pub fn recorder(&self) -> impl Fn(T) + Send + Sync + 'static {
        let output = self.clone();
        move |event| output.with_mut(|events| events.push(event))
    }
}
