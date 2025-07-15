//! # Keyboard State Management
//!
//! This module provides keyboard state management for the Tessera UI framework.
//!
//! ## Overview
//!
//! The keyboard state system manages a bounded queue of keyboard events to ensure
//! efficient event processing and memory management. It acts as a buffer between
//! the windowing system (winit) and the UI framework, allowing for smooth keyboard
//! input handling even during high-frequency key events.
//!
//! ## Design
//!
//! The [`KeyboardState`] struct maintains a bounded queue of keyboard events to ensure:
//! - **Memory efficiency**: Old events are automatically discarded to prevent unbounded growth
//! - **Event ordering**: Events are processed in the order they were received
//! - **Performance**: The queue size is limited to prevent excessive memory usage during rapid key presses
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::KeyboardState;
//! use winit::event::{KeyEvent, ElementState};
//! use winit::keyboard::{KeyCode, PhysicalKey};
//!
//! let mut keyboard_state = KeyboardState::default();
//!
//! // Push keyboard events as they arrive from winit
//! let key_event = KeyEvent {
//!     physical_key: PhysicalKey::Code(KeyCode::KeyA),
//!     logical_key: winit::keyboard::Key::Character("a".into()),
//!     text: Some("a".into()),
//!     location: winit::keyboard::KeyLocation::Standard,
//!     state: ElementState::Pressed,
//!     repeat: false,
//!     platform_specific: Default::default(),
//! };
//! keyboard_state.push_event(key_event);
//!
//! // Process all pending keyboard events
//! let events = keyboard_state.take_events();
//! for event in events {
//!     match event.state {
//!         ElementState::Pressed => {
//!             // Handle key press
//!         }
//!         ElementState::Released => {
//!             // Handle key release
//!         }
//!     }
//! }
//! ```

use std::collections::VecDeque;

/// Maximum number of keyboard events to keep in the queue.
///
/// This constant limits the size of the keyboard event queue to prevent unbounded
/// memory growth during rapid key input. When the queue exceeds this size, the
/// oldest events are automatically removed to make room for new ones.
///
/// The value of 10 is chosen to balance between:
/// - Responsiveness: Ensuring recent events are not lost
/// - Memory efficiency: Preventing excessive memory usage
/// - Performance: Keeping queue operations fast
const KEEP_EVENTS_COUNT: usize = 10;

/// Manages the state and event queue for keyboard input.
///
/// The `KeyboardState` struct provides a bounded queue for storing keyboard events
/// received from the windowing system. It automatically manages memory by discarding
/// old events when the queue becomes too large, ensuring consistent performance
/// even during rapid keyboard input.
///
/// ## Thread Safety
///
/// This struct is not thread-safe by itself and should be protected by appropriate
/// synchronization primitives when used across multiple threads.
///
/// ## Examples
///
/// ```rust,ignore
/// use crate::KeyboardState;
/// use winit::event::KeyEvent;
///
/// let mut keyboard_state = KeyboardState::default();
///
/// // The queue starts empty
/// assert!(keyboard_state.take_events().is_empty());
///
/// // Events can be pushed and retrieved
/// keyboard_state.push_event(key_event);
/// let events = keyboard_state.take_events();
/// assert_eq!(events.len(), 1);
/// ```
#[derive(Default)]
pub struct KeyboardState {
    /// Internal queue storing keyboard events in chronological order.
    ///
    /// Events are added to the back of the queue and removed from the front,
    /// maintaining FIFO (First In, First Out) ordering. The queue is automatically
    /// bounded by [`KEEP_EVENTS_COUNT`] to prevent memory issues.
    events: VecDeque<winit::event::KeyEvent>,
}

impl KeyboardState {
    /// Adds a new keyboard event to the end of the queue.
    ///
    /// This method appends the provided keyboard event to the internal queue.
    /// If adding the event would cause the queue to exceed [`KEEP_EVENTS_COUNT`],
    /// the oldest event is automatically removed to maintain the size limit.
    ///
    /// ## Parameters
    ///
    /// * `event` - The keyboard event to add to the queue. This should be a
    ///   [`winit::event::KeyEvent`] received from the windowing system.
    ///
    /// ## Examples
    ///
    /// ```rust,ignore
    /// use crate::KeyboardState;
    /// use winit::event::{KeyEvent, ElementState};
    /// use winit::keyboard::{KeyCode, PhysicalKey};
    ///
    /// let mut keyboard_state = KeyboardState::default();
    ///
    /// let key_event = KeyEvent {
    ///     physical_key: PhysicalKey::Code(KeyCode::Space),
    ///     logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space),
    ///     text: Some(" ".into()),
    ///     location: winit::keyboard::KeyLocation::Standard,
    ///     state: ElementState::Pressed,
    ///     repeat: false,
    ///     platform_specific: Default::default(),
    /// };
    ///
    /// keyboard_state.push_event(key_event);
    /// ```
    pub fn push_event(&mut self, event: winit::event::KeyEvent) {
        // Add the event to the deque
        self.events.push_back(event);
        // If the events deque is too long, we remove the oldest one
        if self.events.len() > KEEP_EVENTS_COUNT {
            self.events.pop_front();
        }
    }

    /// Removes and returns all keyboard events from the queue.
    ///
    /// This method drains the entire event queue, returning all stored events
    /// as a vector in chronological order (oldest first). After calling this
    /// method, the internal queue will be empty.
    ///
    /// This is typically called during the event processing phase of the UI
    /// framework's update cycle to handle all pending keyboard input.
    ///
    /// ## Returns
    ///
    /// A `Vec<winit::event::KeyEvent>` containing all keyboard events that were
    /// in the queue, ordered from oldest to newest. If the queue was empty,
    /// returns an empty vector.
    ///
    /// ## Examples
    ///
    /// ```rust,ignore
    /// use crate::KeyboardState;
    /// use winit::event::KeyEvent;
    ///
    /// let mut keyboard_state = KeyboardState::default();
    ///
    /// // Add some events
    /// keyboard_state.push_event(key_event1);
    /// keyboard_state.push_event(key_event2);
    ///
    /// // Process all events
    /// let events = keyboard_state.take_events();
    /// assert_eq!(events.len(), 2);
    ///
    /// // Queue is now empty
    /// let empty_events = keyboard_state.take_events();
    /// assert!(empty_events.is_empty());
    /// ```
    pub fn take_events(&mut self) -> Vec<winit::event::KeyEvent> {
        self.events.drain(..).collect()
    }
}
