//! # Keyboard State Management
//!
//! This module provides keyboard state management.

use std::collections::VecDeque;

use winit::keyboard::ModifiersState;

/// Manages the state and event queue for keyboard input.
///
/// The `KeyboardState` struct provides an unbounded queue for storing keyboard
/// events received from the windowing system. Events are retained until they
/// are drained, so a long frame cannot drop key presses or releases.
#[derive(Default, Debug)]
pub struct KeyboardState {
    /// Internal queue storing keyboard events in chronological order.
    ///
    /// Events are added to the back of the queue and removed from the front,
    /// maintaining FIFO (First In, First Out) ordering. The queue is unbounded:
    /// every pushed event is kept until [`Self::take_events`] drains it.
    events: VecDeque<winit::event::KeyEvent>,
    /// Current state of the keyboard modifiers (e.g., Shift, Ctrl, Alt).
    modifiers: ModifiersState,
}

impl KeyboardState {
    /// Adds a new keyboard event to the end of the queue.
    ///
    /// This method appends the provided keyboard event to the internal queue.
    /// The queue is unbounded, so the event is never discarded to make room
    /// for a newer one.
    ///
    /// ## Parameters
    ///
    /// * `event` - The keyboard event to add to the queue. This should be a
    ///   [`winit::event::KeyEvent`] received from the windowing system.
    pub fn push_event(&mut self, event: winit::event::KeyEvent) {
        self.events.push_back(event);
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
