use std::collections::VecDeque;

const KEEP_EVENTS_COUNT: usize = 10;

/// The state of the keyboard
#[derive(Default)]
pub struct KeyboardState {
    /// Keyboard events queue
    events: VecDeque<winit::event::KeyEvent>,
}

impl KeyboardState {
    /// Push a new keyboard event to the queue
    pub fn push_event(&mut self, event: winit::event::KeyEvent) {
        // Add the event to the deque
        self.events.push_back(event);
        // If the events deque is too long, we remove the oldest one
        if self.events.len() > KEEP_EVENTS_COUNT {
            self.events.pop_front();
        }
    }

    /// Take all keyboard events from the queue
    pub fn take_events(&mut self) -> Vec<winit::event::KeyEvent> {
        self.events.drain(..).collect()
    }

    /// Clear all keyboard events
    pub fn clear(&mut self) {
        self.events.clear();
    }
}
