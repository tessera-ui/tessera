//! Cursor state management and event handling system.
//!
//! This module provides comprehensive cursor and touch event handling for the Tessera UI framework.
//! It manages cursor position tracking, event queuing, touch gesture recognition, and inertial
//! scrolling for smooth user interactions.
//!
//! # Key Features
//!
//! - **Multi-touch Support**: Tracks multiple simultaneous touch points with unique IDs
//! - **Inertial Scrolling**: Provides smooth momentum-based scrolling after touch gestures
//! - **Event Queuing**: Maintains a bounded queue of cursor events for processing
//! - **Velocity Tracking**: Calculates touch velocities for natural gesture recognition
//! - **Cross-platform**: Handles both mouse and touch input events consistently
//!
//! # Usage
//!
//! The main entry point is [`CursorState`], which maintains all cursor-related state:
//!
//! ```rust,ignore
//! use tessera_ui::cursor::CursorState;
//! use tessera_ui::PxPosition;
//!
//! let mut cursor_state = CursorState::default();
//!
//! // Handle touch start
//! cursor_state.handle_touch_start(0, PxPosition::new(100.0, 200.0));
//!
//! // Process events
//! let events = cursor_state.take_events();
//! for event in events {
//!     match event.content {
//!         CursorEventContent::Pressed(_) => println!("Touch started"),
//!         CursorEventContent::Scroll(scroll) => {
//!             println!("Scroll: dx={}, dy={}", scroll.delta_x, scroll.delta_y);
//!         }
//!         _ => {}
//!     }
//! }
//! ```

use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use crate::PxPosition;

/// Maximum number of events to keep in the queue to prevent memory issues during UI jank.
const KEEP_EVENTS_COUNT: usize = 10;

/// Controls how quickly inertial scrolling decelerates (higher = faster slowdown).
const INERTIA_DECAY_CONSTANT: f32 = 5.0;

/// Minimum velocity threshold below which inertial scrolling stops (pixels per second).
const MIN_INERTIA_VELOCITY: f32 = 10.0;

/// Minimum velocity from a gesture required to start inertial scrolling (pixels per second).
const INERTIA_MIN_VELOCITY_THRESHOLD_FOR_START: f32 = 50.0;

/// Multiplier applied to initial inertial velocity (typically 1.0 for natural feel).
const INERTIA_MOMENTUM_FACTOR: f32 = 1.0;

/// Tracks the state of a single touch point for gesture recognition and velocity calculation.
///
/// This struct maintains the necessary information to track touch movement, calculate
/// velocities, and determine when to trigger inertial scrolling.
///
/// # Example
///
/// ```rust,ignore
/// let touch_state = TouchPointState {
///     last_position: PxPosition::new(100.0, 200.0),
///     last_update_time: Instant::now(),
///     velocity_tracker: VelocityTracker::new(Instant::now()),
/// };
/// ```
#[derive(Debug, Clone)]
struct TouchPointState {
    /// The last recorded position of this touch point.
    last_position: PxPosition,
    /// Timestamp of the last position update.
    last_update_time: Instant,
    /// Tracks recent velocity samples and temporal metadata for momentum calculation.
    velocity_tracker: VelocityTracker,
    /// Tracks whether this touch gesture generated a scroll event.
    ///
    /// When set, the gesture should be treated as a drag/scroll rather than a tap.
    generated_scroll_event: bool,
}

/// Maintains a short window of velocity samples for inertia calculations.
#[derive(Debug, Clone)]
struct VelocityTracker {
    samples: VecDeque<(Instant, f32, f32)>,
    last_sample_time: Instant,
}

const VELOCITY_SAMPLE_WINDOW: Duration = Duration::from_millis(90);
const VELOCITY_IDLE_CUTOFF: Duration = Duration::from_millis(65);

/// Represents an active inertial scrolling session.
///
/// When a touch gesture ends with sufficient velocity, this struct tracks
/// the momentum and gradually decelerates the scroll movement over time.
///
/// # Example
///
/// ```rust,ignore
/// let inertia = ActiveInertia {
///     velocity_x: 200.0,  // pixels per second
///     velocity_y: -150.0, // pixels per second  
///     last_tick_time: Instant::now(),
/// };
/// ```
#[derive(Debug, Clone)]
struct ActiveInertia {
    /// Current horizontal velocity in pixels per second.
    velocity_x: f32,
    /// Current vertical velocity in pixels per second.
    velocity_y: f32,
    /// Timestamp of the last inertia calculation update.
    last_tick_time: Instant,
}

/// Configuration settings for touch scrolling behavior.
///
/// This struct controls various aspects of how touch gestures are interpreted
/// and converted into scroll events.
///
/// # Example
///
/// ```rust,ignore
/// let config = TouchScrollConfig {
///     min_move_threshold: 3.0,  // More sensitive
///     enabled: true,
/// };
/// ```
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
/// `CursorState` is the main interface for handling all cursor-related events in the Tessera
/// UI framework. It manages cursor position tracking, event queuing, multi-touch support,
/// and provides smooth inertial scrolling for touch gestures.
///
/// # Key Responsibilities
///
/// - **Position Tracking**: Maintains current cursor/touch position
/// - **Event Management**: Queues and processes cursor events with bounded storage
/// - **Multi-touch Support**: Tracks multiple simultaneous touch points
/// - **Inertial Scrolling**: Provides momentum-based scrolling after touch gestures
/// - **Cross-platform Input**: Handles both mouse and touch events uniformly
///
/// # Usage
///
/// ```rust,ignore
/// use tessera_ui::cursor::{CursorState, CursorEventContent};
/// use tessera_ui::PxPosition;
///
/// let mut cursor_state = CursorState::default();
///
/// // Handle a touch gesture
/// cursor_state.handle_touch_start(0, PxPosition::new(100.0, 200.0));
/// cursor_state.handle_touch_move(0, PxPosition::new(110.0, 190.0));
/// cursor_state.handle_touch_end(0);
///
/// // Process accumulated events
/// let events = cursor_state.take_events();
/// for event in events {
///     match event.content {
///         CursorEventContent::Pressed(_) => println!("Touch started"),
///         CursorEventContent::Scroll(scroll) => {
///             println!("Scrolling: dx={}, dy={}", scroll.delta_x, scroll.delta_y);
///         }
///         CursorEventContent::Released(_) => println!("Touch ended"),
///     }
/// }
/// ```
///
/// # Thread Safety
///
/// `CursorState` is not thread-safe and should be used from a single thread,
/// typically the main UI thread where input events are processed.
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
    /// Current inertial scrolling state, if active.
    active_inertia: Option<ActiveInertia>,
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
    /// Events are stored in a bounded queue to prevent memory issues during UI performance
    /// problems. If the queue exceeds [`KEEP_EVENTS_COUNT`], the oldest events are discarded.
    ///
    /// # Arguments
    ///
    /// * `event` - The cursor event to add to the queue
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tessera_ui::cursor::{
    ///     CursorState, CursorEvent, CursorEventContent, GestureState, PressKeyEventType,
    /// };
    /// use std::time::Instant;
    ///
    /// let mut cursor_state = CursorState::default();
    /// let event = CursorEvent {
    ///     timestamp: Instant::now(),
    ///     content: CursorEventContent::Pressed(PressKeyEventType::Left),
    ///     gesture_state: GestureState::TapCandidate,
    /// };
    /// cursor_state.push_event(event);
    /// ```
    pub fn push_event(&mut self, event: CursorEvent) {
        self.events.push_back(event);

        // Maintain bounded queue size to prevent memory issues during UI jank
        if self.events.len() > KEEP_EVENTS_COUNT {
            self.events.pop_front();
        }
    }

    /// Updates the current cursor position.
    ///
    /// This method accepts any type that can be converted into `Option<PxPosition>`,
    /// allowing for flexible position updates including clearing the position by
    /// passing `None`.
    ///
    /// # Arguments
    ///
    /// * `position` - New cursor position or `None` to clear the position
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tessera_ui::cursor::CursorState;
    /// use tessera_ui::PxPosition;
    ///
    /// let mut cursor_state = CursorState::default();
    ///
    /// // Set position
    /// cursor_state.update_position(PxPosition::new(100.0, 200.0));
    ///
    /// // Clear position
    /// cursor_state.update_position(None);
    /// ```
    pub fn update_position(&mut self, position: impl Into<Option<PxPosition>>) {
        self.position = position.into();
    }

    /// Processes active inertial scrolling and generates scroll events.
    ///
    /// This method is called internally to update inertial scrolling state and generate
    /// appropriate scroll events. It handles velocity decay over time and stops inertia
    /// when velocity falls below the minimum threshold.
    ///
    /// The method calculates scroll deltas based on current velocity and elapsed time,
    /// applies exponential decay to the velocity, and queues scroll events for processing.
    ///
    /// # Implementation Details
    ///
    /// - Uses exponential decay with [`INERTIA_DECAY_CONSTANT`] for natural deceleration
    /// - Stops inertia when velocity drops below [`MIN_INERTIA_VELOCITY`]
    /// - Generates scroll events with calculated position deltas
    /// - Handles edge cases like zero delta time gracefully
    fn process_and_queue_inertial_scroll(&mut self) {
        // Handle active inertia with clear, small responsibilities.
        if let Some(mut inertia) = self.active_inertia.take() {
            let now = Instant::now();
            let delta_time = now.duration_since(inertia.last_tick_time).as_secs_f32();

            if delta_time <= 0.0 {
                // Called multiple times in the same instant; reinsert for next frame.
                self.active_inertia = Some(inertia);
                return;
            }

            // Compute scroll delta and emit event if meaningful.
            let scroll_delta_x = inertia.velocity_x * delta_time;
            let scroll_delta_y = inertia.velocity_y * delta_time;
            if scroll_delta_x.abs() > 0.01 || scroll_delta_y.abs() > 0.01 {
                self.push_scroll_event(now, scroll_delta_x, scroll_delta_y);
            }

            // Apply exponential decay to velocities.
            let decay = (-INERTIA_DECAY_CONSTANT * delta_time).exp();
            inertia.velocity_x *= decay;
            inertia.velocity_y *= decay;
            inertia.last_tick_time = now;

            // Reinsert inertia only if still above threshold.
            if inertia.velocity_x.abs() >= MIN_INERTIA_VELOCITY
                || inertia.velocity_y.abs() >= MIN_INERTIA_VELOCITY
            {
                self.active_inertia = Some(inertia);
            }
        }
    }

    // Helper: push a scroll event with consistent construction.
    fn push_scroll_event(&mut self, timestamp: Instant, dx: f32, dy: f32) {
        self.push_event(CursorEvent {
            timestamp,
            content: CursorEventContent::Scroll(ScrollEventConent {
                delta_x: dx,
                delta_y: dy,
            }),
            gesture_state: GestureState::Dragged,
        });
    }

    /// Retrieves and clears all pending cursor events.
    ///
    /// This method processes any active inertial scrolling, then returns all queued
    /// cursor events and clears the internal event queue. Events are returned in
    /// chronological order (oldest first).
    ///
    /// This is typically called once per frame by the UI framework to process
    /// all accumulated input events.
    ///
    /// # Returns
    ///
    /// A vector of [`CursorEvent`]s ordered from oldest to newest.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tessera_ui::cursor::CursorState;
    ///
    /// let mut cursor_state = CursorState::default();
    ///
    /// // ... handle some input events ...
    ///
    /// // Process all events at once
    /// let events = cursor_state.take_events();
    /// for event in events {
    ///     println!("Event at {:?}: {:?}", event.timestamp, event.content);
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// Events are ordered from oldest to newest to ensure proper event processing order.
    pub fn take_events(&mut self) -> Vec<CursorEvent> {
        self.process_and_queue_inertial_scroll();
        self.events.drain(..).collect()
    }

    /// Clears all cursor state and pending events.
    ///
    /// This method resets the cursor state to its initial condition by:
    /// - Clearing all queued events
    /// - Removing cursor position information
    /// - Stopping any active inertial scrolling
    /// - Clearing all touch point tracking
    ///
    /// This is typically used when the UI context changes significantly,
    /// such as when switching between different UI screens or when input
    /// focus changes.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tessera_ui::cursor::CursorState;
    ///
    /// let mut cursor_state = CursorState::default();
    ///
    /// // ... handle various input events ...
    ///
    /// // Reset everything when changing UI context
    /// cursor_state.clear();
    /// ```
    pub fn clear(&mut self) {
        self.events.clear();
        self.update_position(None);
        self.active_inertia = None;
        self.touch_points.clear();
        self.clear_position_on_next_frame = false;
    }

    /// Returns the current cursor position, if any.
    ///
    /// The position represents the last known location of the cursor or active touch point.
    /// Returns `None` if no cursor is currently active or if the position has been cleared.
    ///
    /// # Returns
    ///
    /// - `Some(PxPosition)` if a cursor position is currently tracked
    /// - `None` if no cursor is active
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tessera_ui::cursor::CursorState;
    /// use tessera_ui::PxPosition;
    ///
    /// let mut cursor_state = CursorState::default();
    ///
    /// // Initially no position
    /// assert_eq!(cursor_state.position(), None);
    ///
    /// // After setting position
    /// cursor_state.update_position(PxPosition::new(100.0, 200.0));
    /// assert_eq!(cursor_state.position(), Some(PxPosition::new(100.0, 200.0)));
    /// ```
    pub fn position(&self) -> Option<PxPosition> {
        self.position
    }

    /// Handles the start of a touch gesture.
    ///
    /// This method registers a new touch point and generates a press event. It also
    /// stops any active inertial scrolling since a new touch interaction has begun.
    ///
    /// # Arguments
    ///
    /// * `touch_id` - Unique identifier for this touch point
    /// * `position` - Initial position of the touch in pixel coordinates
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tessera_ui::cursor::CursorState;
    /// use tessera_ui::PxPosition;
    ///
    /// let mut cursor_state = CursorState::default();
    /// cursor_state.handle_touch_start(0, PxPosition::new(100.0, 200.0));
    ///
    /// // This generates a Pressed event and updates the cursor position
    /// let events = cursor_state.take_events();
    /// assert_eq!(events.len(), 1);
    /// ```
    pub fn handle_touch_start(&mut self, touch_id: u64, position: PxPosition) {
        self.active_inertia = None; // Stop any existing inertia on new touch
        let now = Instant::now();

        self.touch_points.insert(
            touch_id,
            TouchPointState {
                last_position: position,
                last_update_time: now,
                velocity_tracker: VelocityTracker::new(now),
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
    /// This method tracks touch movement, calculates velocities for inertial scrolling,
    /// and generates scroll events when the movement exceeds the minimum threshold.
    /// It also maintains a velocity history for momentum calculation.
    ///
    /// # Arguments
    ///
    /// * `touch_id` - Unique identifier for the touch point being moved
    /// * `current_position` - New position of the touch in pixel coordinates
    ///
    /// # Returns
    ///
    /// - `Some(CursorEvent)` containing a scroll event if movement exceeds threshold
    /// - `None` if movement is below threshold or touch scrolling is disabled
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tessera_ui::cursor::CursorState;
    /// use tessera_ui::PxPosition;
    ///
    /// let mut cursor_state = CursorState::default();
    /// cursor_state.handle_touch_start(0, PxPosition::new(100.0, 200.0));
    ///
    /// // Move touch point - may generate scroll event
    /// if let Some(scroll_event) = cursor_state.handle_touch_move(0, PxPosition::new(110.0, 190.0)) {
    ///     println!("Scroll detected!");
    /// }
    /// ```
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
            let time_delta = now
                .duration_since(touch_state.last_update_time)
                .as_secs_f32();

            touch_state.last_position = current_position;
            touch_state.last_update_time = now;

            if move_distance >= self.touch_scroll_config.min_move_threshold {
                // Stop any active inertia when user actively moves the touch.
                self.active_inertia = None;

                if time_delta > 0.0 {
                    let velocity_x = delta_x / time_delta;
                    let velocity_y = delta_y / time_delta;
                    touch_state
                        .velocity_tracker
                        .push(now, velocity_x, velocity_y);
                }

                touch_state.generated_scroll_event = true;

                // Return a scroll event for immediate feedback.
                return Some(CursorEvent {
                    timestamp: now,
                    content: CursorEventContent::Scroll(ScrollEventConent {
                        delta_x, // Direct scroll delta for touch move
                        delta_y,
                    }),
                    gesture_state: GestureState::Dragged,
                });
            }
        }
        None
    }

    /// Handles the end of a touch gesture and potentially starts inertial scrolling.
    ///
    /// This method processes the end of a touch interaction by:
    /// - Calculating average velocity from recent touch movement
    /// - Starting inertial scrolling if velocity exceeds the threshold
    /// - Generating a release event
    /// - Cleaning up touch point tracking
    ///
    /// # Arguments
    ///
    /// * `touch_id` - Unique identifier for the touch point that ended
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tessera_ui::cursor::CursorState;
    /// use tessera_ui::PxPosition;
    ///
    /// let mut cursor_state = CursorState::default();
    /// cursor_state.handle_touch_start(0, PxPosition::new(100.0, 200.0));
    /// cursor_state.handle_touch_move(0, PxPosition::new(150.0, 180.0));
    /// cursor_state.handle_touch_end(0);
    ///
    /// // May start inertial scrolling based on gesture velocity
    /// let events = cursor_state.take_events();
    /// // Events may include scroll events from inertia
    /// ```
    pub fn handle_touch_end(&mut self, touch_id: u64) {
        let now = Instant::now();
        let mut was_drag = false;

        if let Some(touch_state) = self.touch_points.get_mut(&touch_id) {
            was_drag |= touch_state.generated_scroll_event;
            if self.touch_scroll_config.enabled {
                if let Some((avg_vx, avg_vy)) = touch_state.velocity_tracker.resolve(now) {
                    let velocity_magnitude = (avg_vx * avg_vx + avg_vy * avg_vy).sqrt();
                    if velocity_magnitude > INERTIA_MIN_VELOCITY_THRESHOLD_FOR_START {
                        self.active_inertia = Some(ActiveInertia {
                            velocity_x: avg_vx * INERTIA_MOMENTUM_FACTOR,
                            velocity_y: avg_vy * INERTIA_MOMENTUM_FACTOR,
                            last_tick_time: now,
                        });
                    } else {
                        self.active_inertia = None;
                    }
                } else {
                    self.active_inertia = None;
                }
            } else {
                self.active_inertia = None; // Scrolling disabled
            }
        } else {
            self.active_inertia = None; // No touch state present
        }

        if self.active_inertia.is_some() {
            was_drag = true;
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

        if self.touch_points.is_empty() && self.active_inertia.is_none() {
            self.clear_position_on_next_frame = true;
        }
    }
}

impl VelocityTracker {
    fn new(now: Instant) -> Self {
        Self {
            samples: VecDeque::new(),
            last_sample_time: now,
        }
    }

    fn push(&mut self, now: Instant, vx: f32, vy: f32) {
        self.samples.push_back((now, vx, vy));
        self.last_sample_time = now;
        self.prune(now);
    }

    fn resolve(&mut self, now: Instant) -> Option<(f32, f32)> {
        self.prune(now);

        if self.samples.is_empty() {
            return None;
        }

        let idle_time = now.duration_since(self.last_sample_time);
        if idle_time >= VELOCITY_IDLE_CUTOFF {
            self.samples.clear();
            return None;
        }

        let mut weighted_sum_x = 0.0f32;
        let mut weighted_sum_y = 0.0f32;
        let mut total_weight = 0.0f32;
        let window_secs = VELOCITY_SAMPLE_WINDOW.as_secs_f32().max(f32::EPSILON);

        for &(timestamp, vx, vy) in &self.samples {
            let age_secs = now
                .duration_since(timestamp)
                .as_secs_f32()
                .clamp(0.0, window_secs);
            let weight = (window_secs - age_secs).max(0.0);
            if weight > 0.0 {
                weighted_sum_x += vx * weight;
                weighted_sum_y += vy * weight;
                total_weight += weight;
            }
        }

        if total_weight <= f32::EPSILON {
            self.samples.clear();
            return None;
        }

        let mut avg_x = weighted_sum_x / total_weight;
        let mut avg_y = weighted_sum_y / total_weight;

        let damping = 1.0 - idle_time.as_secs_f32() / VELOCITY_IDLE_CUTOFF.as_secs_f32();
        let damping = damping.clamp(0.0, 1.0);
        avg_x *= damping;
        avg_y *= damping;

        Some((avg_x, avg_y))
    }

    fn prune(&mut self, now: Instant) {
        while let Some(&(timestamp, _, _)) = self.samples.front() {
            if now.duration_since(timestamp) > VELOCITY_SAMPLE_WINDOW {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }
}

/// Represents a single cursor or touch event with timing information.
///
/// `CursorEvent` encapsulates all types of cursor interactions including presses,
/// releases, and scroll actions. Each event includes a timestamp for precise
/// timing and ordering of input events.
///
/// # Example
///
/// ```rust,ignore
/// use tessera_ui::cursor::{CursorEvent, CursorEventContent, PressKeyEventType};
/// use std::time::Instant;
///
/// let event = CursorEvent {
///     timestamp: Instant::now(),
///     content: CursorEventContent::Pressed(PressKeyEventType::Left),
/// };
///
/// match event.content {
///     CursorEventContent::Pressed(button) => println!("Button pressed: {:?}", button),
///     CursorEventContent::Released(button) => println!("Button released: {:?}", button),
///     CursorEventContent::Scroll(scroll) => {
///         println!("Scroll: dx={}, dy={}", scroll.delta_x, scroll.delta_y);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CursorEvent {
    /// Timestamp indicating when this event occurred.
    pub timestamp: Instant,
    /// The specific type and data of this cursor event.
    pub content: CursorEventContent,
    /// Classification of the gesture associated with this event.
    ///
    /// Events originating from touch scrolling will mark this as [`GestureState::Dragged`],
    /// allowing downstream components to distinguish tap candidates from scroll gestures.
    pub gesture_state: GestureState,
}

/// Contains scroll movement data for scroll events.
///
/// `ScrollEventConent` represents the amount of scrolling that occurred,
/// with positive values typically indicating rightward/downward movement
/// and negative values indicating leftward/upward movement.
///
/// # Example
///
/// ```rust,ignore
/// use tessera_ui::cursor::ScrollEventConent;
///
/// let scroll = ScrollEventConent {
///     delta_x: 10.0,   // Scroll right 10 pixels
///     delta_y: -20.0,  // Scroll up 20 pixels
/// };
///
/// println!("Horizontal scroll: {}", scroll.delta_x);
/// println!("Vertical scroll: {}", scroll.delta_y);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollEventConent {
    /// Horizontal scroll distance in pixels.
    ///
    /// Positive values indicate rightward scrolling,
    /// negative values indicate leftward scrolling.
    pub delta_x: f32,
    /// Vertical scroll distance in pixels.
    ///
    /// Positive values indicate downward scrolling,
    /// negative values indicate upward scrolling.
    pub delta_y: f32,
}

/// Enumeration of all possible cursor event types.
///
/// `CursorEventContent` represents the different kinds of interactions
/// that can occur with cursor or touch input, including button presses,
/// releases, and scroll actions.
///
/// # Example
///
/// ```rust,ignore
/// use tessera_ui::cursor::{CursorEventContent, PressKeyEventType, ScrollEventConent};
///
/// // Handle different event types
/// match event_content {
///     CursorEventContent::Pressed(PressKeyEventType::Left) => {
///         println!("Left button pressed");
///     }
///     CursorEventContent::Released(PressKeyEventType::Right) => {
///         println!("Right button released");
///     }
///     CursorEventContent::Scroll(scroll) => {
///         println!("Scrolled by ({}, {})", scroll.delta_x, scroll.delta_y);
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum CursorEventContent {
    /// A cursor button or touch point was pressed.
    Pressed(PressKeyEventType),
    /// A cursor button or touch point was released.
    Released(PressKeyEventType),
    /// A scroll action occurred (mouse wheel, touch drag, or inertial scroll).
    Scroll(ScrollEventConent),
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
    /// This method converts winit's mouse button events into Tessera's cursor event format.
    /// It handles the three standard mouse buttons (left, right, middle) and ignores
    /// any additional buttons that may be present on some mice.
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
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tessera_ui::cursor::CursorEventContent;
    /// use winit::event::{ElementState, MouseButton};
    ///
    /// let press_event = CursorEventContent::from_press_event(
    ///     ElementState::Pressed,
    ///     MouseButton::Left
    /// );
    ///
    /// if let Some(event) = press_event {
    ///     println!("Created cursor event: {:?}", event);
    /// }
    /// ```
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
    /// This method converts winit's mouse scroll delta into Tessera's scroll event format.
    /// It handles both line-based scrolling (typical mouse wheels) and pixel-based
    /// scrolling (trackpads, precision mice) by applying appropriate scaling.
    ///
    /// # Arguments
    ///
    /// * `delta` - The scroll delta from winit
    ///
    /// # Returns
    ///
    /// A `CursorEventContent::Scroll` event with scaled delta values.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tessera_ui::cursor::CursorEventContent;
    /// use winit::event::MouseScrollDelta;
    ///
    /// let scroll_event = CursorEventContent::from_scroll_event(
    ///     MouseScrollDelta::LineDelta(0.0, 1.0)  // Scroll down one line
    /// );
    ///
    /// match scroll_event {
    ///     CursorEventContent::Scroll(scroll) => {
    ///         println!("Scroll delta: ({}, {})", scroll.delta_x, scroll.delta_y);
    ///     }
    ///     _ => {}
    /// }
    /// ```
    pub fn from_scroll_event(delta: winit::event::MouseScrollDelta) -> Self {
        let (delta_x, delta_y) = match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
            winit::event::MouseScrollDelta::PixelDelta(delta) => (delta.x as f32, delta.y as f32),
        };

        const MOUSE_WHEEL_SPEED_MULTIPLIER: f32 = 50.0;
        Self::Scroll(ScrollEventConent {
            delta_x: delta_x * MOUSE_WHEEL_SPEED_MULTIPLIER,
            delta_y: delta_y * MOUSE_WHEEL_SPEED_MULTIPLIER,
        })
    }
}

/// Represents the different types of cursor buttons or touch interactions.
///
/// `PressKeyEventType` identifies which button was pressed or released in
/// a cursor event. This covers the three standard mouse buttons that are
/// commonly supported across different platforms and input devices.
///
/// # Example
///
/// ```rust,ignore
/// use tessera_ui::cursor::PressKeyEventType;
///
/// match button_type {
///     PressKeyEventType::Left => println!("Primary button (usually left-click)"),
///     PressKeyEventType::Right => println!("Secondary button (usually right-click)"),
///     PressKeyEventType::Middle => println!("Middle button (usually scroll wheel click)"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PressKeyEventType {
    /// The primary mouse button (typically left button) or primary touch.
    Left,
    /// The secondary mouse button (typically right button).
    Right,
    /// The middle mouse button (typically scroll wheel click).
    Middle,
}
