use std::sync::Arc;
use std::time::Instant;

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera::{
    ComputedData, Constraint, CursorEventContent, DimensionValue, Px, PxPosition,
    focus_state::Focus, measure_node, place_node,
};
use tessera_macros::tessera;

#[derive(Debug, Builder)]
pub struct ScrollableArgs {
    /// The desired width behavior of the scrollable area.
    /// Defaults to Wrap { min: None, max: None }.
    #[builder(default = "tessera::DimensionValue::Wrap { min: None, max: None }")]
    pub width: tessera::DimensionValue,
    /// The desired height behavior of the scrollable area.
    /// Defaults to Wrap { min: None, max: None }.
    #[builder(default = "tessera::DimensionValue::Wrap { min: None, max: None }")]
    pub height: tessera::DimensionValue,
    /// Is vertical scrollable?
    /// Defaults to `true` since most scrollable areas are vertical.
    #[builder(default = "true")]
    pub vertical: bool,
    /// Is horizontal scrollable?
    /// Defaults to `false` since most scrollable areas are not horizontal.
    #[builder(default = "false")]
    pub horizontal: bool,
    /// Scroll speed multiplier. Higher values make scrolling faster.
    /// Defaults to 20.0 for reasonable scroll speed.
    #[builder(default = "20.0")]
    pub scroll_speed: f32,
    /// Scroll smoothing factor (0.0 = instant, 1.0 = very smooth).
    /// Defaults to 0.15 for responsive but smooth scrolling.
    #[builder(default = "0.15")]
    pub scroll_smoothing: f32,
}

/// The state of Scrollable.
pub struct ScrollableState {
    /// The current position of the child component (for rendering)
    child_position: PxPosition,
    /// The target position of the child component (scrolling destination)
    target_position: PxPosition,
    /// The child component size
    child_size: ComputedData,
    /// Last frame time for delta time calculation
    last_frame_time: Option<Instant>,
    /// Focus handler for the scrollable component
    focus_handler: Focus,
}

impl ScrollableState {
    /// Creates a new ScrollableState with default values.
    pub fn new() -> Self {
        Self {
            child_position: PxPosition::ZERO,
            target_position: PxPosition::ZERO,
            child_size: ComputedData::ZERO,
            last_frame_time: None,
            focus_handler: Focus::new(),
        }
    }

    /// Updates the scroll position based on time-based interpolation
    /// Returns true if the position changed (needs redraw)
    fn update_scroll_position(&mut self, smoothing: f32) -> bool {
        let current_time = Instant::now();

        // Calculate delta time
        let delta_time = if let Some(last_time) = self.last_frame_time {
            current_time.duration_since(last_time).as_secs_f32()
        } else {
            0.016 // Assume 60fps for first frame
        };

        self.last_frame_time = Some(current_time);

        // If we're close enough to target, snap to it
        let distance_x = (self.target_position.x - self.child_position.x).abs();
        let distance_y = (self.target_position.y - self.child_position.y).abs();
        let total_distance = distance_x + distance_y;

        if total_distance <= 1 {
            if self.child_position != self.target_position {
                self.child_position = self.target_position;
                return true;
            }
            return false;
        }

        // Calculate interpolation factor based on smoothing and delta time
        // Higher smoothing = more lerp, lower smoothing = more immediate
        let lerp_factor = (1.0 - smoothing).powf(delta_time * 60.0).clamp(0.0, 1.0);

        // Interpolate towards target
        let old_position = self.child_position;
        let delta_x = self.target_position.x.to_f32() - self.child_position.x.to_f32();
        let delta_y = self.target_position.y.to_f32() - self.child_position.y.to_f32();

        self.child_position = PxPosition {
            x: Px::from_f32(self.child_position.x.to_f32() + delta_x * (1.0 - lerp_factor)),
            y: Px::from_f32(self.child_position.y.to_f32() + delta_y * (1.0 - lerp_factor)),
        };

        // Return true if position changed significantly
        old_position != self.child_position
    }

    /// Sets a new target position for scrolling
    fn set_target_position(&mut self, target: PxPosition) {
        self.target_position = target;
    }

    /// Gets a reference to the focus handler
    pub fn focus_handler(&self) -> &Focus {
        &self.focus_handler
    }

    /// Gets a mutable reference to the focus handler
    pub fn focus_handler_mut(&mut self) -> &mut Focus {
        &mut self.focus_handler
    }
}

#[tessera]
pub fn scrollable(
    args: impl Into<ScrollableArgs>,
    state: Arc<RwLock<ScrollableState>>,
    child: impl FnOnce(),
) {
    let args: ScrollableArgs = args.into();
    {
        let state = state.clone();
        measure(Box::new(move |input| {
            // Merge constraints with parent constraints
            let arg_constraint = Constraint {
                width: args.width,
                height: args.height,
            };
            let merged_constraint = input.effective_constraint.merge(&arg_constraint);
            // Now calculate the constraints to child
            let mut child_constraint = merged_constraint.clone();
            // If vertical scrollable, set height to wrap
            if args.vertical {
                child_constraint.height = tessera::DimensionValue::Wrap {
                    min: None,
                    max: None,
                };
            }
            // If horizontal scrollable, set width to wrap
            if args.horizontal {
                child_constraint.width = tessera::DimensionValue::Wrap {
                    min: None,
                    max: None,
                };
            }
            // Measure the child with child constraint
            let child_node_id = input.children_ids[0]; // Scrollable should have exactly one child
            let child_measurement = measure_node(
                child_node_id,
                &child_constraint,
                input.tree,
                input.metadatas,
            )?;
            // Update the child position and size in the state
            state.write().child_size = child_measurement;

            // Update scroll position based on time and get current position for rendering
            let current_child_position = {
                let mut state_guard = state.write();
                state_guard.update_scroll_position(args.scroll_smoothing);
                state_guard.child_position
            };

            // Place child at current interpolated position
            place_node(child_node_id, current_child_position, input.metadatas);
            // Calculate the size of the scrollable area
            let width = match merged_constraint.width {
                DimensionValue::Fixed(w) => w,
                DimensionValue::Wrap { min, max } => {
                    let mut width = child_measurement.width;
                    if let Some(min_width) = min {
                        width = width.max(min_width);
                    }
                    if let Some(max_width) = max {
                        width = width.min(max_width);
                    }
                    width
                }
                DimensionValue::Fill { min: _, max } => max.unwrap(),
            };
            let height = match merged_constraint.height {
                DimensionValue::Fixed(h) => h,
                DimensionValue::Wrap { min, max } => {
                    let mut height = child_measurement.height;
                    if let Some(min_height) = min {
                        height = height.max(min_height)
                    }
                    if let Some(max_height) = max {
                        height = height.min(max_height)
                    }
                    height
                }
                DimensionValue::Fill { min: _, max } => max.unwrap(),
            };
            // Return the size of the scrollable area
            Ok(ComputedData { width, height })
        }));
    }

    // Handle scroll input and position updates
    state_handler(Box::new(move |input| {
        // Update scroll position based on time (always call this each frame)
        state.write().update_scroll_position(args.scroll_smoothing);

        // Handle click events to request focus
        let click_events: Vec<_> = input
            .cursor_events
            .iter()
            .filter(|event| matches!(event.content, CursorEventContent::Pressed(_)))
            .collect();

        if !click_events.is_empty() {
            // Request focus if not already focused
            if !state.read().focus_handler().is_focused() {
                state.write().focus_handler_mut().request_focus();
            }
        }

        // Handle scroll events (only when focused)
        if state.read().focus_handler().is_focused() {
            for event in input
                .cursor_events
                .iter()
                .filter_map(|event| match &event.content {
                    CursorEventContent::Scroll(event) => Some(event),
                    _ => None,
                })
            {
                let mut state_guard = state.write();

                // Apply scroll speed multiplier
                let scroll_delta_x = event.delta_x * args.scroll_speed;
                let scroll_delta_y = event.delta_y * args.scroll_speed;

                // Calculate new target position
                let current_target = state_guard.target_position;
                let new_target = current_target
                    .offset(Px::from_f32(scroll_delta_x), Px::from_f32(scroll_delta_y));

                // Set new target position
                state_guard.set_target_position(new_target);
            }
        }

        // Apply bounds constraints to target position
        let target_position = state.read().target_position;
        let child_size = state.read().child_size;
        let constrained_target = constrain_position(
            target_position,
            &child_size,
            &input.computed_data,
            args.vertical,
            args.horizontal,
        );

        // Set contrained target position
        state.write().set_target_position(constrained_target);
    }));

    // Add child component
    child();
}

/// Constrains a position to stay within the scrollable bounds
fn constrain_position(
    position: PxPosition,
    child_size: &ComputedData,
    container_size: &ComputedData,
    vertical_scrollable: bool,
    horizontal_scrollable: bool,
) -> PxPosition {
    let mut constrained = position;

    // Only apply constraints for scrollable directions
    if horizontal_scrollable {
        // Check if left edge of the child is out of bounds
        if constrained.x > Px::ZERO {
            constrained.x = Px::ZERO;
        }
        // Check if right edge of the child is out of bounds
        if constrained.x + child_size.width < container_size.width {
            constrained.x = container_size.width - child_size.width;
        }
    } else {
        // Not horizontally scrollable, keep at zero
        constrained.x = Px::ZERO;
    }

    if vertical_scrollable {
        // Check if top edge of the child is out of bounds
        if constrained.y > Px::ZERO {
            constrained.y = Px::ZERO;
        }
        // Check if bottom edge of the child is out of bounds
        if constrained.y + child_size.height < container_size.height {
            constrained.y = container_size.height - child_size.height;
        }
    } else {
        // Not vertically scrollable, keep at zero
        constrained.y = Px::ZERO;
    }

    constrained
}
