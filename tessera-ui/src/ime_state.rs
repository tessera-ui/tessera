//! # IME State Management
//!
//! This module provides IME state management.

use std::collections::VecDeque;

/// Former maximum number of IME events kept in the queue.
///
/// This constant is retained for compatibility. The IME event queue is now
/// unbounded: [`ImeState::push_event`] only appends and never truncates, so
/// this value is unused and no longer drops the oldest events.
#[deprecated(note = "IME events are no longer truncated; the queue is unbounded")]
#[allow(dead_code)]
pub const KEEP_EVENTS_COUNT: usize = 10;

/// Manages the state and event queue for Input Method Editor (IME) operations.
///
/// The `ImeState` struct provides an unbounded queue for storing IME events
/// from the windowing system. Events are retained until they are drained, so a
/// long frame cannot drop composition updates or committed text.
#[derive(Default)]
pub(crate) struct ImeState {
    /// Internal queue storing pending IME events.
    ///
    /// Events are stored in the order they were received. New events are added
    /// to the back and stay there until [`ImeState::take_events`] drains the
    /// queue. The queue is unbounded and does not drop the oldest events.
    events: VecDeque<winit::event::Ime>,
}

impl ImeState {
    /// Adds a new IME event to the end of the queue.
    ///
    /// This method appends the provided IME event to the internal queue. The
    /// queue is unbounded, so the event is never discarded to make room for a
    /// newer one.
    ///
    /// # Arguments
    ///
    /// * `event` - The IME event to add to the queue. This can be any variant
    ///   of [`winit::event::Ime`], including composition text, committed text,
    ///   or IME state changes.
    pub fn push_event(&mut self, event: winit::event::Ime) {
        self.events.push_back(event);
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
    /// ordered from oldest to newest. If the queue was empty, returns an empty
    /// vector.
    pub fn take_events(&mut self) -> Vec<winit::event::Ime> {
        self.events.drain(..).collect()
    }
}
