//! # Cursor management
//!
//! This module provides comprehensive cursor and touch event handling for the
//! Tessera UI framework. It manages cursor position tracking, event queuing,
//! touch gesture recognition, and scroll event generation for smooth user
//! interactions.

use std::collections::{HashMap, VecDeque};

use crate::{PxPosition, time::Instant};

/// Pointer identifier used by input changes.
pub type PointerId = u64;
/// Pointer identifier reserved for mouse input.
pub const MOUSE_POINTER_ID: PointerId = 0;

/// Soft queue limit for compacting redundant motion samples during UI jank.
/// Gesture boundaries and scroll deltas are never evicted to meet this limit.
const KEEP_EVENTS_COUNT: usize = 128;

/// Tracks the state of a single touch point for gesture recognition and
/// scroll tracking.
///
/// This struct maintains the necessary information to track touch movement and
/// determine when to trigger scrolling.
#[derive(Debug, Clone)]
struct TouchPointState {
    /// The last recorded position of this touch point.
    last_position: PxPosition,
    /// Initial position used for cumulative touch slop.
    start_position: PxPosition,
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
/// in the Tessera UI framework. It manages cursor position tracking, pointer
/// change queuing, and multi-touch support for touch gestures.
#[derive(Default)]
pub struct CursorState {
    /// Current cursor position, if any cursor is active.
    position: Option<PxPosition>,
    /// Bounded queue of pointer changes awaiting processing.
    events: VecDeque<PointerChange>,
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

    /// Adds a pointer change to the processing queue.
    ///
    /// Motion samples are compacted above a soft queue limit. Scroll deltas
    /// and gesture boundaries are retained even during a long frame.
    ///
    /// # Arguments
    ///
    /// * `event` - The pointer change to add to the queue
    pub fn push_event(&mut self, event: PointerChange) {
        self.events.push_back(event);
        while self.events.len() > KEEP_EVENTS_COUNT {
            let redundant = self.events.iter().enumerate().find_map(|(index, queued)| {
                if !matches!(queued.content, CursorEventContent::Moved(_)) {
                    return None;
                }
                self.events
                    .iter()
                    .skip(index + 1)
                    .find(|next| next.pointer_id == queued.pointer_id)
                    .filter(|next| matches!(next.content, CursorEventContent::Moved(_)))
                    .map(|_| index)
            });
            if let Some(index) = redundant {
                self.events.remove(index);
            } else {
                break;
            }
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

    /// Retrieves and clears all pending pointer changes.
    ///
    /// This method returns all queued pointer changes and clears the internal
    /// event queue. Events are returned in chronological order (oldest first).
    ///
    /// This is typically called once per frame by the UI framework to process
    /// all accumulated input events.
    ///
    /// # Returns
    ///
    /// A vector of [`PointerChange`]s ordered from oldest to newest.
    ///
    /// # Note
    ///
    /// Events are ordered from oldest to newest to ensure proper event
    /// processing order.
    pub fn take_events(&mut self) -> Vec<PointerChange> {
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
                start_position: position,
                last_update_time: now,
                generated_scroll_event: false,
            },
        );
        self.update_position(position);
        let press_event = PointerChange {
            timestamp: now,
            pointer_id: touch_id,
            content: CursorEventContent::Pressed(PressKeyEventType::Left),
            gesture_state: GestureState::TapCandidate,
            consumed: false,
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
    /// - `Some(PointerChange)` containing a scroll event if movement exceeds
    ///   threshold
    /// - `None` if movement is below threshold or touch scrolling is disabled
    pub fn handle_touch_move(
        &mut self,
        touch_id: u64,
        current_position: PxPosition,
    ) -> Option<PointerChange> {
        let now = Instant::now();
        let touch_state = self.touch_points.get_mut(&touch_id)?;
        let mut delta_x = (current_position.x - touch_state.last_position.x).to_f32();
        let mut delta_y = (current_position.y - touch_state.last_position.y).to_f32();
        let cumulative_x = (current_position.x - touch_state.start_position.x).to_f32();
        let cumulative_y = (current_position.y - touch_state.start_position.y).to_f32();
        let distance = cumulative_x.hypot(cumulative_y);
        if self.touch_scroll_config.enabled
            && !touch_state.generated_scroll_event
            && distance >= self.touch_scroll_config.min_move_threshold
        {
            touch_state.generated_scroll_event = true;
            let beyond_slop = (distance - self.touch_scroll_config.min_move_threshold) / distance;
            delta_x = cumulative_x * beyond_slop;
            delta_y = cumulative_y * beyond_slop;
        }
        touch_state.last_position = current_position;
        touch_state.last_update_time = now;
        let dragging = touch_state.generated_scroll_event;
        let gesture_state = if dragging {
            GestureState::Dragged
        } else {
            GestureState::TapCandidate
        };
        self.update_position(current_position);
        self.push_event(PointerChange {
            timestamp: now,
            pointer_id: touch_id,
            content: CursorEventContent::Moved(current_position),
            gesture_state,
            consumed: false,
        });
        (self.touch_scroll_config.enabled && dragging).then_some(PointerChange {
            timestamp: now,
            pointer_id: touch_id,
            content: CursorEventContent::Scroll(ScrollEventContent {
                delta_x,
                delta_y,
                unit: ScrollDeltaUnit::Pixel,
                source: ScrollEventSource::Touch,
            }),
            gesture_state,
            consumed: false,
        })
    }

    /// Ends a touch as a completed gesture and emits a release event.
    pub fn handle_touch_end(&mut self, touch_id: u64) {
        self.finish_touch(touch_id, false);
    }

    /// Cancels a touch gesture. Cancellation never produces a tap or fling.
    pub fn handle_touch_cancel(&mut self, touch_id: u64) {
        self.finish_touch(touch_id, true);
    }

    fn finish_touch(&mut self, touch_id: u64, cancelled: bool) {
        let now = Instant::now();
        let touch_state = match self.touch_points.remove(&touch_id) {
            Some(state) => state,
            None => return,
        };
        let was_drag = touch_state.generated_scroll_event;
        self.update_position(touch_state.last_position);
        self.push_event(PointerChange {
            timestamp: now,
            pointer_id: touch_id,
            content: if cancelled {
                CursorEventContent::Cancelled(PressKeyEventType::Left)
            } else {
                CursorEventContent::Released(PressKeyEventType::Left)
            },
            gesture_state: if cancelled || was_drag {
                GestureState::Dragged
            } else {
                GestureState::TapCandidate
            },
            consumed: false,
        });
        if self.touch_points.is_empty() {
            self.clear_position_on_next_frame = true;
        }
    }
}

/// Represents a single pointer change with timing information.
///
/// `PointerChange` encapsulates all pointer interactions including
/// presses, releases, and scroll actions. Each event includes a timestamp for
/// precise timing and ordering of input events.
#[derive(Debug, Clone)]
pub struct PointerChange {
    /// Timestamp indicating when this event occurred.
    pub timestamp: Instant,
    /// Pointer identifier for this input stream.
    pub pointer_id: PointerId,
    /// The specific type and data of this pointer change.
    pub content: CursorEventContent,
    /// Classification of the gesture associated with this event.
    ///
    /// Events originating from touch scrolling will mark this as
    /// [`GestureState::Dragged`], allowing downstream components to
    /// distinguish tap candidates from scroll gestures.
    pub gesture_state: GestureState,
    /// Whether this change has been consumed by a handler.
    pub(crate) consumed: bool,
}

impl PointerChange {
    /// Creates an unconsumed pointer sample with an explicit event timestamp.
    pub fn new(
        timestamp: Instant,
        pointer_id: PointerId,
        content: CursorEventContent,
        gesture_state: GestureState,
    ) -> Self {
        Self {
            timestamp,
            pointer_id,
            content,
            gesture_state,
            consumed: false,
        }
    }
    /// Marks this change as consumed.
    pub fn consume(&mut self) {
        self.consumed = true;
    }

    /// Returns whether this change has already been consumed.
    pub fn is_consumed(&self) -> bool {
        self.consumed
    }
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
    /// Unit used by the originating platform event.
    pub unit: ScrollDeltaUnit,
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
    /// The pointer moved to a new absolute position.
    Moved(PxPosition),
    /// A cursor button or touch point was pressed.
    Pressed(PressKeyEventType),
    /// A cursor button or touch point was released.
    Released(PressKeyEventType),
    /// A touch point was cancelled by the platform.
    Cancelled(PressKeyEventType),
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
    /// event format while preserving whether the platform reported the value
    /// in line or pixel units.
    ///
    /// # Arguments
    ///
    /// * `delta` - The scroll delta from winit
    ///
    /// # Returns
    ///
    /// A `CursorEventContent::Scroll` event with raw line or pixel delta
    /// values.
    pub fn from_scroll_event(delta: winit::event::MouseScrollDelta) -> Self {
        let (delta_x, delta_y, unit) = match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y, ScrollDeltaUnit::Line),
            winit::event::MouseScrollDelta::PixelDelta(delta) => {
                (delta.x as f32, delta.y as f32, ScrollDeltaUnit::Pixel)
            }
        };

        Self::Scroll(ScrollEventContent {
            delta_x,
            delta_y,
            unit,
            source: ScrollEventSource::Wheel,
        })
    }
}

/// Represents the different types of cursor buttons or touch interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Indicates the unit used by the platform to describe a scroll delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDeltaUnit {
    /// Delta is expressed as a logical number of lines or wheel steps.
    Line,
    /// Delta is expressed in pixels.
    Pixel,
}

#[cfg(test)]
mod queue_tests {
    use super::*;
    use crate::Px;
    #[test]
    fn long_frame_preserves_touch_boundaries_and_displacement() {
        let mut cursor = CursorState::default();
        cursor.handle_touch_start(1, PxPosition::ZERO);
        for x in 1..=300 {
            if let Some(scroll) = cursor.handle_touch_move(1, PxPosition::new(Px(x), Px(0))) {
                cursor.push_event(scroll);
            }
        }
        cursor.handle_touch_end(1);
        let events = cursor.take_events();
        assert!(matches!(events[0].content, CursorEventContent::Pressed(_)));
        assert!(matches!(
            events.last().unwrap().content,
            CursorEventContent::Released(_)
        ));
        let distance: f32 = events
            .iter()
            .filter_map(|event| match &event.content {
                CursorEventContent::Scroll(scroll) => Some(scroll.delta_x),
                _ => None,
            })
            .sum();
        assert!((distance - 295.0).abs() < 0.001);
    }
}
#[cfg(test)]
mod tests {
    use super::{CursorEventContent, CursorState, GestureState, PxPosition};
    use crate::Px;

    fn position(x: i32) -> PxPosition {
        PxPosition::new(Px::new(x), Px::ZERO)
    }

    #[test]
    fn touch_slop_is_cumulative_and_follow_up_micro_moves_scroll() {
        let mut cursor = CursorState::default();
        cursor.handle_touch_start(1, position(0));
        assert!(cursor.handle_touch_move(1, position(2)).is_none());
        assert!(cursor.handle_touch_move(1, position(4)).is_none());
        let first = cursor
            .handle_touch_move(1, position(6))
            .expect("slop crossed");
        assert_eq!(first.gesture_state, GestureState::Dragged);
        let second = cursor
            .handle_touch_move(1, position(7))
            .expect("drag remains active");
        assert_eq!(second.gesture_state, GestureState::Dragged);
        assert!(
            matches!(second.content, CursorEventContent::Scroll(scroll) if scroll.delta_x == 1.0)
        );
    }

    #[test]
    fn cancelled_touch_is_not_a_tap_boundary() {
        let mut cursor = CursorState::default();
        cursor.handle_touch_start(1, position(0));
        cursor.handle_touch_cancel(1);
        let events = cursor.take_events();
        assert!(matches!(
            events.last().map(|event| &event.content),
            Some(CursorEventContent::Cancelled(_))
        ));
        assert_eq!(
            events.last().map(|event| event.gesture_state),
            Some(GestureState::Dragged)
        );
    }
}
