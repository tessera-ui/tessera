use std::{collections::VecDeque, time::Instant};

// We don't want to keep too many events in the queue
// when ui is janked(in badly way!)
const KEEP_EVENTS_COUNT: usize = 10;

/// The state of the cursor
#[derive(Default)]
pub struct CursorState {
    /// Tracks the cursor position
    ///
    /// # For mouse
    /// `None` means the cursor is out of the window
    ///
    /// # For touch
    ///
    /// `None` means user is not touching the screen
    position: Option<[i32; 2]>,
    /// Press event deque
    events: VecDeque<CursorEvent>,
}

impl CursorState {
    /// Push cursor event to queue
    pub fn push_event(&mut self, event: CursorEvent) {
        // Add the event to the deque
        self.events.push_back(event);
        // If the events deque is too long, we remove the oldest one
        if self.events.len() > KEEP_EVENTS_COUNT {
            self.events.pop_front();
        }
    }

    /// Update the cursor position in state
    pub fn update_position(&mut self, position: impl Into<Option<[i32; 2]>>) {
        self.position = position.into();
    }

    /// Custom a group of events
    ///
    /// # Note: Events are ordered from left (oldest) to right (newest)
    pub fn take_events(&mut self) -> Vec<CursorEvent> {
        self.events.drain(..).collect()
    }

    /// Clear all cursor events
    pub fn clear(&mut self) {
        self.events.clear();
        self.update_position(None);
    }

    /// Get the current cursor position
    pub fn position(&self) -> Option<[i32; 2]> {
        self.position
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

/// Event representing a scroll action
#[derive(Debug, Clone)]
pub struct ScrollEventType {
    /// Horizontal scroll delta
    pub delta_x: f32,
    /// Vertical scroll delta
    pub delta_y: f32,
}

/// Cursor event types
#[derive(Debug, Clone)]
pub enum CursorEventContent {
    /// The cursor is pressed
    Pressed(PressKeyEventType),
    /// The cursor is released
    Released(PressKeyEventType),
    /// The cursor is scrolled
    Scroll(ScrollEventType),
}

impl CursorEventContent {
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

    /// Create a scroll event
    pub fn from_scroll_event(delta: winit::event::MouseScrollDelta) -> Self {
        let (delta_x, delta_y) = match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
            winit::event::MouseScrollDelta::PixelDelta(delta) => (delta.x as f32, delta.y as f32),
        };
        Self::Scroll(ScrollEventType { delta_x, delta_y })
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
