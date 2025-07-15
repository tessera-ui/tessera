//! # Focus State Management
//!
//! This module provides focus state management for the Tessera UI framework.
//!
//! ## Overview
//!
//! Focus is a common requirement in UI frameworks. As a functional UI framework,
//! `tessera` lacks a stable method for locating specific components by reference.
//! Treating focus as an independent, shared state aligns better with `tessera`'s
//! design philosophy and provides greater flexibility.
//!
//! ## Design Philosophy
//!
//! The focus system is designed around the following principles:
//! - **Decentralized**: Each component can create its own [`Focus`] instance
//! - **Thread-safe**: Focus state is managed using atomic operations and locks
//! - **Automatic cleanup**: Focus is automatically cleared when a [`Focus`] instance is dropped
//! - **Unique identification**: Each focus instance has a unique UUID to prevent conflicts
//!
//! ## Usage
//!
//! ```
//! use tessera_ui::Focus;
//!
//! // Create a new focus instance
//! let focus = Focus::new();
//!
//! // Check if this focus is currently active
//! if focus.is_focused() {
//!     // Handle focused state
//! }
//!
//! // Request focus for this component
//! focus.request_focus();
//!
//! // Clear focus when no longer needed
//! focus.unfocus();
//! ```
//!
//! ## Thread Safety
//!
//! The focus state is managed through a global static variable protected by
//! read-write locks, making it safe to use across multiple threads. This is
//! essential for Tessera's parallelized design.

use std::sync::OnceLock;

use parking_lot::{RwLock, RwLockReadGuard};
use uuid::Uuid;

/// Global focus state storage.
///
/// This static variable holds the shared focus state across the entire application.
/// It's initialized lazily on first access and protected by a read-write lock
/// for thread-safe access.
static FOCUS_STATE: OnceLock<RwLock<FocusState>> = OnceLock::new();

/// Internal focus state representation.
///
/// This structure holds the current focus state, tracking which component
/// (if any) currently has focus through its unique identifier.
#[derive(Default)]
struct FocusState {
    /// The UUID of the currently focused component, or `None` if no component has focus.
    focused: Option<Uuid>,
}

/// Acquires a read lock on the global focus state.
///
/// This function provides thread-safe read access to the focus state.
/// Multiple readers can access the state simultaneously, but writers
/// will be blocked until all readers are finished.
///
/// # Returns
///
/// A read guard that provides access to the focus state. The guard
/// automatically releases the lock when dropped.
fn read_focus_state() -> RwLockReadGuard<'static, FocusState> {
    FOCUS_STATE
        .get_or_init(|| RwLock::new(FocusState::default()))
        .read()
}

/// Acquires a write lock on the global focus state.
///
/// This function provides thread-safe write access to the focus state.
/// Only one writer can access the state at a time, and all readers
/// will be blocked until the writer is finished.
///
/// # Returns
///
/// A write guard that provides mutable access to the focus state.
/// The guard automatically releases the lock when dropped.
fn write_focus_state() -> parking_lot::RwLockWriteGuard<'static, FocusState> {
    FOCUS_STATE
        .get_or_init(|| RwLock::new(FocusState::default()))
        .write()
}

/// A focus handle that represents a focusable component.
///
/// Each `Focus` instance has a unique identifier and can be used to manage
/// focus state for a specific component. The focus system ensures that only
/// one component can have focus at a time across the entire application.
///
/// # Examples
///
/// ```
/// use tessera_ui::Focus;
///
/// // Create a focus instance for a component
/// let button_focus = Focus::new();
///
/// // Request focus for this component
/// button_focus.request_focus();
///
/// // Check if this component currently has focus
/// if button_focus.is_focused() {
///     println!("Button is focused!");
/// }
///
/// // Focus is automatically cleared when the instance is dropped
/// drop(button_focus);
/// ```
pub struct Focus {
    /// Unique identifier for this focus instance.
    ///
    /// This UUID is used to distinguish between different focus instances
    /// and ensure that focus operations are applied to the correct component.
    id: Uuid,
}

impl Default for Focus {
    /// Creates a new focus instance with a unique identifier.
    ///
    /// This is equivalent to calling [`Focus::new()`].
    fn default() -> Self {
        Self::new()
    }
}

impl Focus {
    /// Creates a new focus instance with a unique identifier.
    ///
    /// Each focus instance is assigned a unique UUID that distinguishes it
    /// from all other focus instances in the application. This ensures that
    /// focus operations are applied to the correct component.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Focus;
    ///
    /// let focus1 = Focus::new();
    /// let focus2 = Focus::new();
    ///
    /// // There can only be one focused component at a time
    ///
    /// focus1.request_focus();
    /// assert!(focus1.is_focused());
    /// assert!(!focus2.is_focused());
    ///
    /// focus2.request_focus();
    /// assert!(!focus1.is_focused());
    /// assert!(focus2.is_focused());
    /// ```
    pub fn new() -> Self {
        Focus { id: Uuid::new_v4() }
    }

    /// Checks if this focus instance currently has focus.
    ///
    /// Returns `true` if this specific focus instance is the currently
    /// active focus in the global focus state, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Focus;
    ///
    /// let focus = Focus::new();
    ///
    /// // Initially, no focus is active
    /// assert!(!focus.is_focused());
    ///
    /// // After requesting focus, this instance should be focused
    /// focus.request_focus();
    /// assert!(focus.is_focused());
    /// ```
    ///
    /// # Thread Safety
    ///
    /// This method is thread-safe and can be called from any thread.
    /// It acquires a read lock on the global focus state.
    pub fn is_focused(&self) -> bool {
        let focus_state = read_focus_state();
        focus_state.focused == Some(self.id)
    }

    /// Requests focus for this component.
    ///
    /// This method sets the global focus state to this focus instance,
    /// potentially removing focus from any previously focused component.
    /// Only one component can have focus at a time.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Focus;
    ///
    /// let focus1 = Focus::new();
    /// let focus2 = Focus::new();
    ///
    /// focus1.request_focus();
    /// assert!(focus1.is_focused());
    /// assert!(!focus2.is_focused());
    ///
    /// // Requesting focus for focus2 removes it from focus1
    /// focus2.request_focus();
    /// assert!(!focus1.is_focused());
    /// assert!(focus2.is_focused());
    /// ```
    ///
    /// # Thread Safety
    ///
    /// This method is thread-safe and can be called from any thread.
    /// It acquires a write lock on the global focus state.
    pub fn request_focus(&self) {
        let mut focus_state = write_focus_state();
        focus_state.focused = Some(self.id);
    }

    /// Clears focus if this instance currently has it.
    ///
    /// This method removes focus from the global state, but only if this
    /// specific focus instance is the one that currently has focus.
    /// If another component has focus, this method has no effect.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Focus;
    ///
    /// let focus1 = Focus::new();
    /// let focus2 = Focus::new();
    ///
    /// focus1.request_focus();
    /// assert!(focus1.is_focused());
    ///
    /// // Clear focus from focus1
    /// focus1.unfocus();
    /// assert!(!focus1.is_focused());
    ///
    /// // If focus2 has focus, focus1.unfocus() won't affect it
    /// focus2.request_focus();
    /// focus1.unfocus(); // No effect
    /// assert!(focus2.is_focused());
    /// ```
    ///
    /// # Thread Safety
    ///
    /// This method is thread-safe and can be called from any thread.
    /// It acquires a write lock on the global focus state.
    pub fn unfocus(&self) {
        let mut focus_state = write_focus_state();
        if focus_state.focused == Some(self.id) {
            focus_state.focused = None;
        }
    }
}

impl Drop for Focus {
    /// Automatically clears focus when the `Focus` instance is dropped.
    ///
    /// This ensures that focus is properly cleaned up when a component
    /// is destroyed or goes out of scope. If this focus instance currently
    /// has focus, it will be cleared from the global focus state.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Focus;
    ///
    /// {
    ///     let focus = Focus::new();
    ///     focus.request_focus();
    ///     assert!(focus.is_focused());
    /// } // focus is dropped here, automatically clearing focus
    ///
    /// // Focus is now cleared globally
    /// let another_focus = Focus::new();
    /// assert!(!another_focus.is_focused());
    /// ```
    ///
    /// # Thread Safety
    ///
    /// This method is thread-safe and will properly handle cleanup
    /// even if called from different threads.
    fn drop(&mut self) {
        self.unfocus(); // Ensure focus is cleared when the Focus instance is dropped
    }
}
