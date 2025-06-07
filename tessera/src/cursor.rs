use std::{collections::VecDeque, time::Instant};

// We don't want to keep too many events in the queue
// when ui is janked(in badly way!)
const KEEP_DUEQUES_COUNT: usize = 10;
const KEEP_EVENTS_COUNT: usize = 10;

/// The state of the cursor
#[derive(Default)]
pub struct CursorState {
    /// Press event deque
    events: VecDeque<VecDeque<CursorEvent>>,
}

impl CursorState {
    pub fn push_event(&mut self, event: CursorEvent) {
        // Add the event to the deque
        // here we just add the event to the last deque in the deques
        // in order to handle all events in one frame
        if let Some(last_events) = self.events.back_mut() {
            last_events.push_back(event);
            // if the last deque is too long, we remove the oldest one
            if last_events.len() > KEEP_EVENTS_COUNT {
                last_events.pop_front();
            }
        } else {
            self.events.push_back(vec![event].into());
        }
        // If the events deque is too long, we remove the oldest one
        if self.events.len() > KEEP_DUEQUES_COUNT {
            self.events.pop_front();
        }
    }

    /// Custom a group of events
    pub fn pop_events(&mut self) -> Option<VecDeque<CursorEvent>> {
        // Get the last group of events
        self.events.pop_front()
    }

    /// Called when the cursor is moved out of the window
    /// Resets the press info of all keys
    /// and set position to `None`
    pub fn out_of_window(&mut self) {
        self.events.clear();
    }
}

/// Respents a cursor event
#[derive(Debug, Clone)]
pub struct CursorEvent {
    /// when it happened
    pub timestamp: Instant,
    /// event content
    pub content: CursorEventContent,
}

/// Cursor event types
#[derive(Debug, Clone)]
pub enum CursorEventContent {
    /// The cursor is moved
    Moved {
        /// Position, in pixels
        pos: [u32; 2],
    },
    /// The cursor is left the window
    Left,
    /// The cursor is pressed
    Pressed(PressKeyEventType),
    /// The cursor is released
    Released(PressKeyEventType),
}

impl CursorEventContent {
    /// Create a move event
    pub fn from_position(pos: [u32; 2]) -> Self {
        Self::Moved { pos }
    }

    /// Transform the position to be relative to the given position
    /// , if the event contains a position info.
    pub fn into_relative_position(self, abs_start_pos: [u32; 2]) -> Self {
        match self {
            Self::Moved { pos } => Self::Moved {
                pos: [pos[0] - abs_start_pos[0], pos[1] - abs_start_pos[1]],
            },
            Self::Left => Self::Left,
            Self::Pressed(event_type) => Self::Pressed(event_type),
            Self::Released(event_type) => Self::Released(event_type),
        }
    }

    /// Create a key press/release event
    pub fn from_press_event(
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    ) -> Option<Self> {
        let event_type = match button {
            winit::event::MouseButton::Left => PressKeyEventType::Left,
            winit::event::MouseButton::Right => PressKeyEventType::Right,
            winit::event::MouseButton::Middle => PressKeyEventType::Middle,
            _ => return None, // Ignore other buttons
        };
        let state = match state {
            winit::event::ElementState::Pressed => Self::Pressed(event_type),
            winit::event::ElementState::Released => Self::Released(event_type),
        };
        Some(state)
    }
}

/// Event representing a key press or release
#[derive(Debug, Clone)]
pub enum PressKeyEventType {
    /// The left key
    Left,
    /// The right key
    Right,
    /// The middle key
    Middle,
}
