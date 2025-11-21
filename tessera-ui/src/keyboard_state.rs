//! # Keyboard State Management
//!
//! This module provides keyboard state management.

use std::collections::VecDeque;

use winit::keyboard::ModifiersState;

/// Maximum number of keyboard events to keep in the queue.
///
/// This constant limits the size of the keyboard event queue to prevent unbounded
/// memory growth during rapid key input. When the queue exceeds this size, the
/// oldest events are automatically removed to make room for new ones.
///
/// The value of 10 is chosen to balance between:
///
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
#[derive(Default, Debug)]
pub struct KeyboardState {
    /// Internal queue storing keyboard events in chronological order.
    ///
    /// Events are added to the back of the queue and removed from the front,
    /// maintaining FIFO (First In, First Out) ordering. The queue is automatically
    /// bounded by [`KEEP_EVENTS_COUNT`] to prevent memory issues.
    events: VecDeque<winit::event::KeyEvent>,
    /// Current state of the keyboard modifiers (e.g., Shift, Ctrl, Alt).
    modifiers: ModifiersState,
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
    pub fn take_events(&mut self) -> Vec<winit::event::KeyEvent> {
        self.events.drain(..).collect()
    }

    /// Updates the current state of the keyboard modifiers.
    ///
    /// This should be called whenever a `ModifiersChanged` event is received
    /// from the windowing system.
    pub fn update_modifiers(&mut self, new_state: ModifiersState) {
        self.modifiers = new_state;
    }

    /// Returns the current state of the keyboard modifiers.
    pub fn modifiers(&self) -> ModifiersState {
        self.modifiers
    }
}
