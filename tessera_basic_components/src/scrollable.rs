use std::sync::Arc;

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera::{
    ComputedData, Constraint, CursorEventContent, DimensionValue, Px, PxPosition, measure_node,
    place_node,
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
}

/// The state of Scrollable.
pub struct ScrollableState {
    /// The position of the child component
    child_position: PxPosition,
    /// The child component size
    child_size: ComputedData,
}

impl ScrollableState {
    /// Creates a new ScrollableState with default values.
    pub fn new() -> Self {
        Self {
            child_position: PxPosition::ZERO,
            child_size: ComputedData::ZERO,
        }
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
            // Place child at spec position
            place_node(child_node_id, state.read().child_position, input.metadatas);
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

    // Handle scroll input
    state_handler(Box::new(move |input| {
        // Update the child position based on scroll events
        for event in input
            .cursor_events
            .iter()
            .filter_map(|event| match &event.content {
                CursorEventContent::Scroll(event) => Some(event),
                _ => None,
            })
        {
            let prev_child_position = state.read().child_position;
            state.write().child_position = prev_child_position
                .offset(Px::from_f32(event.delta_x), Px::from_f32(event.delta_y));
            println!(
                "Scrollable state updated: {:?}",
                state.read().child_position
            );
        }
        // Restrict the child position to the scrollable area
        // The rule is:
        // The left edge of the child should not go bigger than the left edge of the scrollable area
        // The top edge of the child should not go bigger than top edge of the scrollable area
        // The right edge of the child should not go smaller than right edge of the scrollable area
        let child_position = state.read().child_position;
        let child_size = state.read().child_size;
        let self_size = input.computed_data;
        // Check if left edge of the child is out of bounds
        if child_position.x > Px::ZERO {
            state.write().child_position.x = Px::ZERO;
        }
        // Check if top edge of the child is out of bounds
        if child_position.y > Px::ZERO {
            state.write().child_position.y = Px::ZERO;
        }
        // Check if right edge of the child is out of bounds
        if child_position.x + child_size.width < self_size.width {
            state.write().child_position.x = self_size.width - child_size.width;
        }
        // Check if bottom edge of the child is out of bounds
        if child_position.y + child_size.height < self_size.height {
            state.write().child_position.y = self_size.height - child_size.height;
        }
    }));

    // Add child component
    child();
}
