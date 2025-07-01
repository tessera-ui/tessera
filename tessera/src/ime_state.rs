use std::collections::VecDeque;

const KEEP_EVENTS_COUNT: usize = 10;

/// The state of the IME
#[derive(Default)]
pub struct ImeState {
    /// IME events queue
    events: VecDeque<winit::event::Ime>,
}

impl ImeState {
    /// Push a new IME event to the queue
    pub fn push_event(&mut self, event: winit::event::Ime) {
        // Add the event to the deque
        self.events.push_back(event);
        // If the events deque is too long, we remove the oldest one
        if self.events.len() > KEEP_EVENTS_COUNT {
            self.events.pop_front();
        }
    }

    /// Take all IME events from the queue
    pub fn take_events(&mut self) -> Vec<winit::event::Ime> {
        self.events.drain(..).collect()
    }
}
