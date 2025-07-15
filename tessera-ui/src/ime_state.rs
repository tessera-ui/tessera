//! # Input Method Editor (IME) State Management
//!
//! This module provides IME state management for the Tessera UI framework.
//!
//! ## Overview
//!
//! Input Method Editor (IME) support is essential for handling complex text input,
//! particularly for languages that require composition (such as Chinese, Japanese,
//! Korean, and others). The IME system allows users to input characters through
//! a multi-step process where intermediate composition states are displayed before
//! the final text is committed.
//!
//! ## Design
//!
//! The [`ImeState`] struct maintains a bounded queue of IME events to ensure:
//! - **Memory efficiency**: Old events are automatically discarded to prevent unbounded growth
//! - **Event ordering**: Events are processed in the order they were received
//! - **Performance**: The queue size is limited to prevent excessive memory usage
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::ImeState;
//! use winit::event::Ime;
//!
//! let mut ime_state = ImeState::default();
//!
//! // Push IME events as they arrive
//! ime_state.push_event(Ime::Preedit("hello".to_string(), None));
//! ime_state.push_event(Ime::Commit("world".to_string()));
//!
//! // Process all pending events
//! let events = ime_state.take_events();
//! for event in events {
//!     match event {
//!         Ime::Preedit(text, cursor) => {
//!             // Handle composition text with optional cursor position
//!         }
//!         Ime::Commit(text) => {
//!             // Handle committed text
//!         }
//!         Ime::Enabled => {
//!             // IME was enabled
//!         }
//!         Ime::Disabled => {
//!             // IME was disabled
//!         }
//!     }
//! }
//! ```

use std::collections::VecDeque;

/// Maximum number of IME events to keep in the queue.
///
/// This constant limits the size of the event queue to prevent unbounded memory growth.
/// When the queue exceeds this size, the oldest events are automatically removed.
/// The value of 10 provides a reasonable balance between memory usage and ensuring
/// that recent events are not lost during high-frequency input scenarios.
pub const KEEP_EVENTS_COUNT: usize = 10;

/// Manages the state and event queue for Input Method Editor (IME) operations.
///
/// The `ImeState` struct provides a bounded queue for storing IME events from the
/// windowing system. It automatically manages memory by discarding old events when
/// the queue becomes too large, ensuring consistent performance even during
/// intensive text input sessions.
///
/// ## Thread Safety
///
/// This struct is not thread-safe by itself. If you need to share IME state across
/// threads, wrap it in appropriate synchronization primitives like `Arc<Mutex<ImeState>>`.
///
/// ## Memory Management
///
/// The internal queue automatically maintains a maximum size of [`KEEP_EVENTS_COUNT`]
/// events. When new events are added beyond this limit, the oldest events are
/// automatically removed to prevent unbounded memory growth.
#[derive(Default)]
pub(crate) struct ImeState {
    /// Internal queue storing pending IME events.
    ///
    /// Events are stored in the order they were received, with new events
    /// added to the back and old events removed from the front when the
    /// queue size limit is exceeded.
    events: VecDeque<winit::event::Ime>,
}

impl ImeState {
    /// Adds a new IME event to the end of the queue.
    ///
    /// This method appends the provided IME event to the internal queue. If adding
    /// this event would cause the queue to exceed [`KEEP_EVENTS_COUNT`], the oldest
    /// event is automatically removed to maintain the size limit.
    ///
    /// # Arguments
    ///
    /// * `event` - The IME event to add to the queue. This can be any variant of
    ///   [`winit::event::Ime`], including composition text, committed text, or
    ///   IME state changes.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use crate::ImeState;
    /// use winit::event::Ime;
    ///
    /// let mut ime_state = ImeState::default();
    ///
    /// // Add a composition event
    /// ime_state.push_event(Ime::Preedit("hello".to_string(), Some(5)));
    ///
    /// // Add a commit event
    /// ime_state.push_event(Ime::Commit("world".to_string()));
    /// ```
    pub fn push_event(&mut self, event: winit::event::Ime) {
        // Add the event to the back of the deque
        self.events.push_back(event);

        // Maintain the queue size limit by removing the oldest event if necessary
        if self.events.len() > KEEP_EVENTS_COUNT {
            self.events.pop_front();
        }
    }

    /// Removes and returns all IME events from the queue.
    ///
    /// This method drains the entire event queue, returning all stored events
    /// in the order they were added (oldest first). After calling this method,
    /// the internal queue will be empty.
    ///
    /// This is typically called during the event processing phase of the UI
    /// update cycle to handle all pending IME events at once.
    ///
    /// # Returns
    ///
    /// A `Vec<winit::event::Ime>` containing all events that were in the queue,
    /// ordered from oldest to newest. If the queue was empty, returns an empty vector.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use crate::ImeState;
    /// use winit::event::Ime;
    ///
    /// let mut ime_state = ImeState::default();
    /// ime_state.push_event(Ime::Enabled);
    /// ime_state.push_event(Ime::Preedit("test".to_string(), None));
    /// ime_state.push_event(Ime::Commit("test".to_string()));
    ///
    /// // Process all events
    /// let events = ime_state.take_events();
    /// assert_eq!(events.len(), 3);
    ///
    /// // Queue is now empty
    /// let empty_events = ime_state.take_events();
    /// assert_eq!(empty_events.len(), 0);
    /// ```
    pub fn take_events(&mut self) -> Vec<winit::event::Ime> {
        self.events.drain(..).collect()
    }
}
