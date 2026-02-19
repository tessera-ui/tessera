//! # Cursor management
//!
//! This module provides comprehensive cursor and touch event handling for the
//! Tessera UI framework. It manages cursor position tracking, event queuing,
//! touch gesture recognition, and scroll event generation for smooth user
//! interactions.

use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};

use crate::PxPosition;

/// Maximum number of events to keep in the queue to prevent memory issues
/// during UI jank.
const KEEP_EVENTS_COUNT: usize = 10;

/// Tracks the state of a single touch point for gesture recognition and
/// scroll tracking.
///
/// This struct maintains the necessary information to track touch movement and
/// determine when to trigger scrolling.
#[derive(Debug, Clone)]
struct TouchPointState {
    /// The last recorded position of this touch point.
    last_position: PxPosition,
    /// Timestamp of the last position update.
    last_update_time: Instant,
    /// Tracks whether this touch gesture generated a scroll event.
    ///
    /// When set, the gesture should be treated as a drag/scroll rather than a
    /// tap.
    generated_scroll_event: bool,
}

/// Configuration settings for touch scrolling behavior.
///
/// This struct controls various aspects of how touch gestures are interpreted
/// and converted into scroll events.
#[derive(Debug, Clone)]
struct TouchScrollConfig {
    /// Minimum movement distance in pixels required to trigger a scroll event.
    ///
    /// Smaller values make scrolling more sensitive but may cause jitter.
    /// Larger values require more deliberate movement but provide stability.
    min_move_threshold: f32,
    /// Whether touch scrolling is currently enabled.
    enabled: bool,
}

impl Default for TouchScrollConfig {
    fn default() -> Self {
        Self {
            // Reduced threshold for more responsive touch
            min_move_threshold: 5.0,
            enabled: true,
        }
    }
}

/// Central state manager for cursor and touch interactions.
///
/// `CursorState` is the main interface for handling all cursor-related events
/// in the Tessera UI framework. It manages cursor position tracking, event
/// queuing, and multi-touch support for touch gestures.
#[derive(Default)]
pub struct CursorState {
    /// Current cursor position, if any cursor is active.
    position: Option<PxPosition>,
    /// Bounded queue of cursor events awaiting processing.
    events: VecDeque<CursorEvent>,
    /// Active touch points mapped by their unique touch IDs.
    touch_points: HashMap<u64, TouchPointState>,
    /// Configuration settings for touch scrolling behavior.
    touch_scroll_config: TouchScrollConfig,
    /// If true, the cursor position will be cleared on the next frame.
    clear_position_on_next_frame: bool,
}

impl CursorState {
    /// Cleans up the cursor state at the end of a frame.
    pub(crate) fn frame_cleanup(&mut self) {
        if self.clear_position_on_next_frame {
            self.update_position(None);
            self.clear_position_on_next_frame = false;
        }
    }

    /// Adds a cursor event to the processing queue.
    ///
    /// Events are stored in a bounded queue to prevent memory issues during UI
    /// performance problems. If the queue exceeds [`KEEP_EVENTS_COUNT`],
    /// the oldest events are discarded.
    ///
    /// # Arguments
    ///
    /// * `event` - The cursor event to add to the queue
    pub fn push_event(&mut self, event: CursorEvent) {
        self.events.push_back(event);

        // Maintain bounded queue size to prevent memory issues during UI jank
        if self.events.len() > KEEP_EVENTS_COUNT {
            self.events.pop_front();
        }
    }

    /// Updates the current cursor position.
    ///
    /// This method accepts any type that can be converted into
    /// `Option<PxPosition>`, allowing for flexible position updates
    /// including clearing the position by passing `None`.
    ///
    /// # Arguments
    ///
    /// * `position` - New cursor position or `None` to clear the position
    pub fn update_position(&mut self, position: impl Into<Option<PxPosition>>) {
        self.position = position.into();
    }

    /// Retrieves and clears all pending cursor events.
    ///
    /// This method returns all queued cursor events and clears the internal
    /// event queue. Events are returned in chronological order (oldest first).
    ///
    /// This is typically called once per frame by the UI framework to process
    /// all accumulated input events.
    ///
    /// # Returns
    ///
    /// A vector of [`CursorEvent`]s ordered from oldest to newest.
    ///
    /// # Note
    ///
    /// Events are ordered from oldest to newest to ensure proper event
    /// processing order.
    pub fn take_events(&mut self) -> Vec<CursorEvent> {
        self.events.drain(..).collect()
    }

    /// Clears all cursor state and pending events.
    ///
    /// This is typically used when the UI context changes significantly,
    /// such as when switching between different UI screens or when input
    /// focus changes.
    pub fn clear(&mut self) {
        self.events.clear();
        self.update_position(None);
        self.touch_points.clear();
        self.clear_position_on_next_frame = false;
    }

    /// Returns the current cursor position, if any.
    ///
    /// The position represents the last known location of the cursor or active
    /// touch point. Returns `None` if no cursor is currently active or if
    /// the position has been cleared.
    ///
    /// # Returns
    ///
    /// - `Some(PxPosition)` if a cursor position is currently tracked
    /// - `None` if no cursor is active
    pub fn position(&self) -> Option<PxPosition> {
        self.position
    }

    /// Handles the start of a touch gesture.
    ///
    /// This method registers a new touch point and generates a press event.
    ///
    /// # Arguments
    ///
    /// * `touch_id` - Unique identifier for this touch point
    /// * `position` - Initial position of the touch in pixel coordinates
    pub fn handle_touch_start(&mut self, touch_id: u64, position: PxPosition) {
        self.clear_position_on_next_frame = false;
        let now = Instant::now();

        self.touch_points.insert(
            touch_id,
            TouchPointState {
                last_position: position,
                last_update_time: now,
                generated_scroll_event: false,
            },
        );
        self.update_position(position);
        let press_event = CursorEvent {
            timestamp: now,
            content: CursorEventContent::Pressed(PressKeyEventType::Left),
            gesture_state: GestureState::TapCandidate,
        };
        self.push_event(press_event);
    }

    /// Handles touch movement and generates scroll events when appropriate.
    ///
    /// This method tracks touch movement and generates scroll events when the
    /// movement exceeds the minimum threshold.
    ///
    /// # Arguments
    ///
    /// * `touch_id` - Unique identifier for the touch point being moved
    /// * `current_position` - New position of the touch in pixel coordinates
    ///
    /// # Returns
    ///
    /// - `Some(CursorEvent)` containing a scroll event if movement exceeds
    ///   threshold
    /// - `None` if movement is below threshold or touch scrolling is disabled
    pub fn handle_touch_move(
        &mut self,
        touch_id: u64,
        current_position: PxPosition,
    ) -> Option<CursorEvent> {
        let now = Instant::now();
        self.update_position(current_position);

        if !self.touch_scroll_config.enabled {
            return None;
        }

        if let Some(touch_state) = self.touch_points.get_mut(&touch_id) {
            let delta_x = (current_position.x - touch_state.last_position.x).to_f32();
            let delta_y = (current_position.y - touch_state.last_position.y).to_f32();
            let move_distance = (delta_x * delta_x + delta_y * delta_y).sqrt();
            touch_state.last_position = current_position;
            touch_state.last_update_time = now;

            if move_distance >= self.touch_scroll_config.min_move_threshold {
                touch_state.generated_scroll_event = true;

                // Return a scroll event for immediate feedback.
                return Some(CursorEvent {
                    timestamp: now,
                    content: CursorEventContent::Scroll(ScrollEventContent {
                        delta_x, // Direct scroll delta for touch move
                        delta_y,
                        source: ScrollEventSource::Touch,
                    }),
                    gesture_state: GestureState::Dragged,
                });
            }
        }
        None
    }

    /// Handles the end of a touch gesture and emits a release event.
    ///
    /// This method processes the end of a touch interaction by:
    /// - Determining whether the gesture was a drag
    /// - Generating a release event
    /// - Cleaning up touch point tracking
    ///
    /// # Arguments
    ///
    /// * `touch_id` - Unique identifier for the touch point that ended
    pub fn handle_touch_end(&mut self, touch_id: u64) {
        let now = Instant::now();
        let mut was_drag = false;

        if let Some(touch_state) = self.touch_points.get_mut(&touch_id) {
            was_drag |= touch_state.generated_scroll_event;
        }

        self.touch_points.remove(&touch_id);
        let release_event = CursorEvent {
            timestamp: now,
            content: CursorEventContent::Released(PressKeyEventType::Left),
            gesture_state: if was_drag {
                GestureState::Dragged
            } else {
                GestureState::TapCandidate
            },
        };
        self.push_event(release_event);

        if self.touch_points.is_empty() {
            self.clear_position_on_next_frame = true;
        }
    }
}

/// Represents a single cursor or touch event with timing information.
///
/// `CursorEvent` encapsulates all types of cursor interactions including
/// presses, releases, and scroll actions. Each event includes a timestamp for
/// precise timing and ordering of input events.
#[derive(Debug, Clone)]
pub struct CursorEvent {
    /// Timestamp indicating when this event occurred.
    pub timestamp: Instant,
    /// The specific type and data of this cursor event.
    pub content: CursorEventContent,
    /// Classification of the gesture associated with this event.
    ///
    /// Events originating from touch scrolling will mark this as
    /// [`GestureState::Dragged`], allowing downstream components to
    /// distinguish tap candidates from scroll gestures.
    pub gesture_state: GestureState,
}

/// Contains scroll movement data for scroll events.
///
/// `ScrollEventContent` represents the amount of scrolling that occurred,
/// with positive values typically indicating rightward/downward movement
/// and negative values indicating leftward/upward movement.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollEventContent {
    /// Horizontal scroll distance in pixels.
    pub delta_x: f32,
    /// Vertical scroll distance in pixels.
    pub delta_y: f32,
    /// The input source that produced the scroll event.
    pub source: ScrollEventSource,
}

/// Enumeration of all possible cursor event types.
///
/// `CursorEventContent` represents the different kinds of interactions
/// that can occur with cursor or touch input, including button presses,
/// releases, and scroll actions.
#[derive(Debug, Clone, PartialEq)]
pub enum CursorEventContent {
    /// A cursor button or touch point was pressed.
    Pressed(PressKeyEventType),
    /// A cursor button or touch point was released.
    Released(PressKeyEventType),
    /// A scroll action occurred (mouse wheel or touch drag).
    Scroll(ScrollEventContent),
}

/// Describes the high-level gesture classification of a cursor event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GestureState {
    /// Indicates the event is part of a potential tap/click interaction.
    #[default]
    TapCandidate,
    /// Indicates the event happened during a drag/scroll gesture.
    Dragged,
}

impl CursorEventContent {
    /// Creates a cursor press/release event from winit mouse button events.
    ///
    /// This method converts winit's mouse button events into Tessera's cursor
    /// event format. It handles the three standard mouse buttons (left,
    /// right, middle) and ignores any additional buttons that may be
    /// present on some mice.
    ///
    /// # Arguments
    ///
    /// * `state` - Whether the button was pressed or released
    /// * `button` - Which mouse button was affected
    ///
    /// # Returns
    ///
    /// - `Some(CursorEventContent)` for supported mouse buttons
    /// - `None` for unsupported mouse buttons
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

    /// Creates a scroll event from winit mouse wheel events.
    ///
    /// This method converts winit's mouse scroll delta into Tessera's scroll
    /// event format. It handles both line-based scrolling (typical mouse
    /// wheels) and pixel-based scrolling (trackpads, precision mice) by
    /// applying appropriate scaling.
    ///
    /// # Arguments
    ///
    /// * `delta` - The scroll delta from winit
    ///
    /// # Returns
    ///
    /// A `CursorEventContent::Scroll` event with scaled delta values.
    pub fn from_scroll_event(delta: winit::event::MouseScrollDelta) -> Self {
        let (delta_x, delta_y) = match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
            winit::event::MouseScrollDelta::PixelDelta(delta) => (delta.x as f32, delta.y as f32),
        };

        const MOUSE_WHEEL_SPEED_MULTIPLIER: f32 = 50.0;
        Self::Scroll(ScrollEventContent {
            delta_x: delta_x * MOUSE_WHEEL_SPEED_MULTIPLIER,
            delta_y: delta_y * MOUSE_WHEEL_SPEED_MULTIPLIER,
            source: ScrollEventSource::Wheel,
        })
    }
}

/// Represents the different types of cursor buttons or touch interactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PressKeyEventType {
    /// The primary mouse button (typically left button) or primary touch.
    Left,
    /// The secondary mouse button (typically right button).
    Right,
    /// The middle mouse button (typically scroll wheel click).
    Middle,
}

/// Indicates the input source for a scroll event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollEventSource {
    /// Scroll generated from a touch drag gesture.
    Touch,
    /// Scroll generated by a mouse wheel or trackpad.
    Wheel,
}
