use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};

use crate::PxPosition;

// We don't want to keep too many events in the queue
// when ui is janked(in badly way!)
const KEEP_EVENTS_COUNT: usize = 10;

// Inertia Constants
const INERTIA_DECAY_CONSTANT: f32 = 5.0; // Higher value = faster slowdown
const MIN_INERTIA_VELOCITY: f32 = 10.0; // Pixels per second, below this inertia stops
const INERTIA_MIN_VELOCITY_THRESHOLD_FOR_START: f32 = 50.0; // Min velocity from gesture to start inertia
const INERTIA_MOMENTUM_FACTOR: f32 = 1.0; // Multiplier for initial inertial velocity (usually 1.0)

/// Single touch point tracking state
#[derive(Debug, Clone)]
struct TouchPointState {
    /// Last recorded position
    last_position: PxPosition,
    /// Last update time
    last_update_time: Instant,
    /// Recent velocity tracking for momentum calculation
    velocity_history: VecDeque<(Instant, f32, f32)>, // (time, delta_x, delta_y)
}

/// Stores the state of an active touch scroll inertia.
#[derive(Debug, Clone)]
struct ActiveInertia {
    velocity_x: f32,
    velocity_y: f32,
    last_tick_time: Instant,
}

/// Touch scroll configuration
#[derive(Debug, Clone)]
struct TouchScrollConfig {
    /// Minimum movement threshold in pixels
    min_move_threshold: f32,
    /// Whether touch scrolling is enabled
    enabled: bool,
}

impl Default for TouchScrollConfig {
    fn default() -> Self {
        Self {
            min_move_threshold: 5.0, // Reduced threshold for more responsive touch
            enabled: true,
        }
    }
}

/// The state of the cursor
#[derive(Default)]
pub struct CursorState {
    /// Tracks the cursor position
    position: Option<PxPosition>,
    /// Press event deque
    events: VecDeque<CursorEvent>,
    /// Touch points indexed by touch ID
    touch_points: HashMap<u64, TouchPointState>,
    /// Touch scroll configuration
    touch_scroll_config: TouchScrollConfig,
    /// Active touch scroll inertia state
    active_inertia: Option<ActiveInertia>,
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
    pub fn update_position(&mut self, position: impl Into<Option<PxPosition>>) {
        self.position = position.into();
    }

    /// Processes active touch inertia and queues scroll events if necessary.
    fn process_and_queue_inertial_scroll(&mut self) {
        if let Some(mut inertia_data) = self.active_inertia.take() {
            // Take ownership
            let now = Instant::now();
            let delta_time = now
                .duration_since(inertia_data.last_tick_time)
                .as_secs_f32();

            let mut should_reinsert_inertia = true;

            if delta_time > 0.0 {
                let scroll_delta_x = inertia_data.velocity_x * delta_time;
                let scroll_delta_y = inertia_data.velocity_y * delta_time;

                if scroll_delta_x.abs() > 0.01 || scroll_delta_y.abs() > 0.01 {
                    self.push_event(CursorEvent {
                        // This is now fine
                        timestamp: now,
                        content: CursorEventContent::Scroll(ScrollEventConent {
                            delta_x: scroll_delta_x,
                            delta_y: scroll_delta_y,
                        }),
                    });
                }

                let decay_multiplier = (-INERTIA_DECAY_CONSTANT * delta_time).exp();
                inertia_data.velocity_x *= decay_multiplier;
                inertia_data.velocity_y *= decay_multiplier;
                inertia_data.last_tick_time = now;

                if inertia_data.velocity_x.abs() < MIN_INERTIA_VELOCITY
                    && inertia_data.velocity_y.abs() < MIN_INERTIA_VELOCITY
                {
                    should_reinsert_inertia = false; // Stop inertia
                }
            } else {
                // delta_time is zero or negative, reinsert without modification for next frame.
                // This can happen if called multiple times in the same instant.
            }

            if should_reinsert_inertia {
                self.active_inertia = Some(inertia_data); // Put it back if still active
            }
        }
    }

    /// Custom a group of events
    ///
    /// # Note: Events are ordered from left (oldest) to right (newest)
    pub fn take_events(&mut self) -> Vec<CursorEvent> {
        self.process_and_queue_inertial_scroll();
        self.events.drain(..).collect()
    }

    /// Clear all cursor events
    pub fn clear(&mut self) {
        self.events.clear();
        self.update_position(None);
        self.active_inertia = None; // Also clear active inertia
    }

    /// Get the current cursor position
    pub fn position(&self) -> Option<PxPosition> {
        self.position
    }

    /// Handle touch start event
    pub fn handle_touch_start(&mut self, touch_id: u64, position: PxPosition) {
        self.active_inertia = None; // Stop any existing inertia on new touch
        let now = Instant::now();

        self.touch_points.insert(
            touch_id,
            TouchPointState {
                last_position: position,
                last_update_time: now,
                velocity_history: VecDeque::new(),
            },
        );
        self.update_position(position);
        let press_event = CursorEvent {
            timestamp: now,
            content: CursorEventContent::Pressed(PressKeyEventType::Left),
        };
        self.push_event(press_event);
    }

    /// Handle touch move event, returns possible scroll event
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

            if move_distance >= self.touch_scroll_config.min_move_threshold {
                self.active_inertia = None; // Stop inertia if significant movement occurs

                let time_delta = now
                    .duration_since(touch_state.last_update_time)
                    .as_secs_f32();
                if time_delta > 0.0 {
                    let velocity_x = delta_x / time_delta;
                    let velocity_y = delta_y / time_delta;
                    touch_state
                        .velocity_history
                        .push_back((now, velocity_x, velocity_y));
                    while let Some(&(sample_time, _, _)) = touch_state.velocity_history.front() {
                        if now.duration_since(sample_time).as_millis() > 100 {
                            touch_state.velocity_history.pop_front();
                        } else {
                            break;
                        }
                    }
                }

                touch_state.last_position = current_position;
                touch_state.last_update_time = now;

                return Some(CursorEvent {
                    timestamp: now,
                    content: CursorEventContent::Scroll(ScrollEventConent {
                        delta_x, // Direct scroll delta for touch move
                        delta_y,
                    }),
                });
            }
        }
        None
    }

    /// Handle touch end event
    pub fn handle_touch_end(&mut self, touch_id: u64) {
        let now = Instant::now();

        if let Some(touch_state) = self.touch_points.get(&touch_id) {
            if !touch_state.velocity_history.is_empty() && self.touch_scroll_config.enabled {
                let mut avg_velocity_x = 0.0;
                let mut avg_velocity_y = 0.0;
                let sample_count = touch_state.velocity_history.len();

                for (_, vx, vy) in &touch_state.velocity_history {
                    avg_velocity_x += vx;
                    avg_velocity_y += vy;
                }

                if sample_count > 0 {
                    avg_velocity_x /= sample_count as f32;
                    avg_velocity_y /= sample_count as f32;
                }

                let velocity_magnitude =
                    (avg_velocity_x * avg_velocity_x + avg_velocity_y * avg_velocity_y).sqrt();

                if velocity_magnitude > INERTIA_MIN_VELOCITY_THRESHOLD_FOR_START {
                    self.active_inertia = Some(ActiveInertia {
                        velocity_x: avg_velocity_x * INERTIA_MOMENTUM_FACTOR,
                        velocity_y: avg_velocity_y * INERTIA_MOMENTUM_FACTOR,
                        last_tick_time: now,
                    });
                } else {
                    self.active_inertia = None; // Ensure inertia is cleared if not starting
                }
            } else {
                self.active_inertia = None; // Ensure inertia is cleared
            }
        } else {
            self.active_inertia = None; // Ensure inertia is cleared if touch_state is None
        }

        self.touch_points.remove(&touch_id);
        let release_event = CursorEvent {
            timestamp: now,
            content: CursorEventContent::Released(PressKeyEventType::Left),
        };
        self.push_event(release_event);

        if self.touch_points.is_empty() {
            if self.active_inertia.is_none() {
                self.update_position(None);
            }
        }
    }

    /// Configure touch scroll parameters
    pub fn configure_touch_scroll(&mut self, enabled: bool, min_threshold: f32) {
        self.touch_scroll_config = TouchScrollConfig {
            enabled,
            min_move_threshold: min_threshold,
        };
    }

    /// Get current active touch count
    pub fn active_touch_count(&self) -> usize {
        self.touch_points.len()
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
pub struct ScrollEventConent {
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
    Scroll(ScrollEventConent),
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

    /// Create a scroll event with mouse wheel speed multiplier
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
