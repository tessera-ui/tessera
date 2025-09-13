//! Scrollable container component for Tessera UI.
//!
//! This module provides a scrollable container that enables vertical and/or horizontal scrolling
//! for overflowing content within a UI layout. It is designed as a fundamental building block
//! for creating areas where content may exceed the visible bounds, such as lists, panels, or
//! custom scroll regions.
//!
//! Features include configurable scroll directions, smooth animated scrolling, and stateful
//! management of scroll position and focus. The scrollable area is highly customizable via
//! [`ScrollableArgs`], and integrates with the Tessera UI state management system.
//!
//! Typical use cases include scrollable lists, text areas, image galleries, or any UI region
//! where content may not fit within the allocated space.
//!
//! # Example
//! See [`scrollable()`] for usage details and code samples.
mod scrollbar;
use std::{sync::Arc, time::Instant};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, Px, PxPosition,
    tessera,
};

use crate::{
    alignment::Alignment,
    boxed::{BoxedArgsBuilder, boxed},
    pos_misc::is_position_in_component,
    scrollable::scrollbar::{ScrollBarArgs, ScrollBarState, scrollbar_h, scrollbar_v},
};

#[derive(Debug, Builder, Clone)]
pub struct ScrollableArgs {
    /// The desired width behavior of the scrollable area
    /// Defaults to Wrap { min: None, max: None }.
    #[builder(default = "tessera_ui::DimensionValue::Wrap { min: None, max: None }")]
    pub width: tessera_ui::DimensionValue,
    /// The desired height behavior of the scrollable area.
    /// Defaults to Wrap { min: None, max: None }.
    #[builder(default = "tessera_ui::DimensionValue::Wrap { min: None, max: None }")]
    pub height: tessera_ui::DimensionValue,
    /// Is vertical scrollable?
    /// Defaults to `true` since most scrollable areas are vertical.
    #[builder(default = "true")]
    pub vertical: bool,
    /// Is horizontal scrollable?
    /// Defaults to `false` since most scrollable areas are not horizontal.
    #[builder(default = "false")]
    pub horizontal: bool,
    /// Scroll smoothing factor (0.0 = instant, 1.0 = very smooth).
    /// Defaults to 0.05 for very responsive but still smooth scrolling.
    #[builder(default = "0.05")]
    pub scroll_smoothing: f32,
    /// The behavior of the scrollbar visibility.
    #[builder(default = "ScrollBarBehavior::AlwaysVisible")]
    pub scrollbar_behavior: ScrollBarBehavior,
    /// The color of the scrollbar track.
    #[builder(default = "Color::new(0.0, 0.0, 0.0, 0.1)")]
    pub scrollbar_track_color: Color,
    /// The color of the scrollbar thumb.
    #[builder(default = "Color::new(0.0, 0.0, 0.0, 0.3)")]
    pub scrollbar_thumb_color: Color,
    /// The color of the scrollbar thumb when hovered.
    #[builder(default = "Color::new(0.0, 0.0, 0.0, 0.5)")]
    pub scrollbar_thumb_hover_color: Color,
    /// The layout of the scrollbar relative to the content.
    #[builder(default = "ScrollBarLayout::Alongside")]
    pub scrollbar_layout: ScrollBarLayout,
}

/// Defines the behavior of the scrollbar visibility.
#[derive(Debug, Clone)]
pub enum ScrollBarBehavior {
    /// The scrollbar is always visible.
    AlwaysVisible,
    /// The scrollbar is only visible when scrolling.
    AutoHide,
    /// No scrollbar at all.
    Hidden,
}

/// Defines the layout of the scrollbar relative to the scrollable content.
#[derive(Debug, Clone)]
pub enum ScrollBarLayout {
    /// The scrollbar is placed alongside the content (takes up space in the layout).
    Alongside,
    /// The scrollbar is overlaid on top of the content (doesn't take up space).
    Overlay,
}

impl Default for ScrollableArgs {
    fn default() -> Self {
        ScrollableArgsBuilder::default().build().unwrap()
    }
}

/// Holds the state for a `scrollable` component, managing scroll position and interaction.
///
/// It tracks the current and target scroll positions, the size of the scrollable content, and focus state.
///
/// The scroll position is smoothly interpolated over time to create a fluid scrolling effect.
#[derive(Default)]
pub struct ScrollableState {
    /// The inner state containing scroll position, size
    inner: Arc<RwLock<ScrollableStateInner>>,
    /// The state for vertical scrollbar
    scrollbar_state_v: Arc<RwLock<ScrollBarState>>,
    /// The state for horizontal scrollbar
    scrollbar_state_h: Arc<RwLock<ScrollBarState>>,
}

impl ScrollableState {
    /// Creates a new `ScrollableState` with default values.
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug)]
struct ScrollableStateInner {
    /// The current position of the child component (for rendering)
    child_position: PxPosition,
    /// The target position of the child component (scrolling destination)
    target_position: PxPosition,
    /// The child component size
    child_size: ComputedData,
    /// The visible area size
    visible_size: ComputedData,
    /// Last frame time for delta time calculation
    last_frame_time: Option<Instant>,
}

impl Default for ScrollableStateInner {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollableStateInner {
    /// Creates a new ScrollableState with default values.
    pub fn new() -> Self {
        Self {
            child_position: PxPosition::ZERO,
            target_position: PxPosition::ZERO,
            child_size: ComputedData::ZERO,
            visible_size: ComputedData::ZERO,
            last_frame_time: None,
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

        // Calculate the difference between target and current position
        let diff_x = self.target_position.x.to_f32() - self.child_position.x.to_f32();
        let diff_y = self.target_position.y.to_f32() - self.child_position.y.to_f32();

        // If we're close enough to target, snap to it
        if diff_x.abs() < 1.0 && diff_y.abs() < 1.0 {
            if self.child_position != self.target_position {
                self.child_position = self.target_position;
                return true;
            }
            return false;
        }

        // Use simple velocity-based movement for consistent behavior
        // Higher smoothing = slower movement
        let mut movement_factor = (1.0 - smoothing) * delta_time * 60.0;

        // CRITICAL FIX: Clamp the movement factor to a maximum of 1.0.
        // A factor greater than 1.0 causes the interpolation to overshoot the target,
        // leading to oscillations that grow exponentially, causing the value explosion
        // and overflow panic seen in the logs. Clamping ensures stability by
        // preventing the position from moving past the target in a single frame.
        if movement_factor > 1.0 {
            movement_factor = 1.0;
        }
        let old_position = self.child_position;

        self.child_position = PxPosition {
            x: Px::saturating_from_f32(self.child_position.x.to_f32() + diff_x * movement_factor),
            y: Px::saturating_from_f32(self.child_position.y.to_f32() + diff_y * movement_factor),
        };

        // Return true if position changed significantly
        old_position != self.child_position
    }

    /// Sets a new target position for scrolling
    fn set_target_position(&mut self, target: PxPosition) {
        self.target_position = target;
    }
}

/// A container that makes its content scrollable when it exceeds the container's size.
///
/// The `scrollable` component is a fundamental building block for creating areas with
/// content that may not fit within the allocated space. It supports vertical and/or
/// horizontal scrolling, which can be configured via `ScrollableArgs`.
///
/// The component offers two scrollbar layout options:
/// - `Alongside`: Scrollbars take up space in the layout alongside the content
/// - `Overlay`: Scrollbars are overlaid on top of the content without taking up space
///
/// State management is handled by `ScrollableState`, which must be provided to persist
/// the scroll position across recompositions. The scrolling behavior is animated with
/// a configurable smoothing factor for a better user experience.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use parking_lot::RwLock;
/// use tessera_ui::{DimensionValue, Dp};
/// use tessera_ui_basic_components::{
///     column::{column, ColumnArgs},
///     scrollable::{scrollable, ScrollableArgs, ScrollableState, ScrollBarLayout},
///     text::text,
/// };
///
/// // In a real app, you would manage the state.
/// let scrollable_state = Arc::new(ScrollableState::new());
///
/// // Example with alongside scrollbars (default)
/// scrollable(
///     ScrollableArgs {
///         height: DimensionValue::Fixed(Dp(100.0).into()),
///         scrollbar_layout: ScrollBarLayout::Alongside,
///         ..Default::default()
///     },
///     scrollable_state.clone(),
///     || {
///         column(ColumnArgs::default(), |scope| {
///             scope.child(|| text("Item 1".to_string()));
///             scope.child(|| text("Item 2".to_string()));
///             scope.child(|| text("Item 3".to_string()));
///             scope.child(|| text("Item 4".to_string()));
///             scope.child(|| text("Item 5".to_string()));
///             scope.child(|| text("Item 6".to_string()));
///             scope.child(|| text("Item 7".to_string()));
///             scope.child(|| text("Item 8".to_string()));
///             scope.child(|| text("Item 9".to_string()));
///             scope.child(|| text("Item 10".to_string()));
///         });
///     },
/// );
///
/// // Example with overlay scrollbars
/// scrollable(
///     ScrollableArgs {
///         height: DimensionValue::Fixed(Dp(100.0).into()),
///         scrollbar_layout: ScrollBarLayout::Overlay,
///         ..Default::default()
///     },
///     scrollable_state,
///     || {
///         column(ColumnArgs::default(), |scope| {
///             scope.child(|| text("Item 1".to_string()));
///             scope.child(|| text("Item 2".to_string()));
///             scope.child(|| text("Item 3".to_string()));
///             scope.child(|| text("Item 4".to_string()));
///             scope.child(|| text("Item 5".to_string()));
///             scope.child(|| text("Item 6".to_string()));
///             scope.child(|| text("Item 7".to_string()));
///             scope.child(|| text("Item 8".to_string()));
///             scope.child(|| text("Item 9".to_string()));
///             scope.child(|| text("Item 10".to_string()));
///         });
///     },
/// );
/// ```
///
/// # Panics
///
/// This component will panic if it does not have exactly one child.
///
/// # Arguments
///
/// * `args`: An instance of `ScrollableArgs` or `ScrollableArgsBuilder` to configure the
///   scrollable area's behavior, such as dimensions and scroll directions.
/// * `state`: An `Arc<RwLock<ScrollableState>>` to hold and manage the component's state.
/// * `child`: A closure that defines the content to be placed inside the scrollable container.
///   This closure is executed once to build the component tree.
#[tessera]
pub fn scrollable(
    args: impl Into<ScrollableArgs>,
    state: Arc<ScrollableState>,
    child: impl FnOnce() + Send + Sync + 'static,
) {
    let args: ScrollableArgs = args.into();

    // Create separate ScrollBarArgs for vertical and horizontal scrollbars
    let scrollbar_args_v = ScrollBarArgs {
        total: state.inner.read().child_size.height,
        visible: state.inner.read().visible_size.height,
        offset: state.inner.read().child_position.y,
        thickness: Dp(8.0), // Default scrollbar thickness
        state: state.inner.clone(),
        scrollbar_behavior: args.scrollbar_behavior.clone(),
        track_color: args.scrollbar_track_color,
        thumb_color: args.scrollbar_thumb_color,
        thumb_hover_color: args.scrollbar_thumb_hover_color,
    };

    let scrollbar_args_h = ScrollBarArgs {
        total: state.inner.read().child_size.width,
        visible: state.inner.read().visible_size.width,
        offset: state.inner.read().child_position.x,
        thickness: Dp(8.0), // Default scrollbar thickness
        state: state.inner.clone(),
        scrollbar_behavior: args.scrollbar_behavior.clone(),
        track_color: args.scrollbar_track_color,
        thumb_color: args.scrollbar_thumb_color,
        thumb_hover_color: args.scrollbar_thumb_hover_color,
    };

    match args.scrollbar_layout {
        ScrollBarLayout::Alongside => {
            scrollable_with_alongside_scrollbar(
                state,
                args,
                scrollbar_args_v,
                scrollbar_args_h,
                child,
            );
        }
        ScrollBarLayout::Overlay => {
            scrollable_with_overlay_scrollbar(
                state,
                args,
                scrollbar_args_v,
                scrollbar_args_h,
                child,
            );
        }
    }
}

#[tessera]
fn scrollable_with_alongside_scrollbar(
    state: Arc<ScrollableState>,
    args: ScrollableArgs,
    scrollbar_args_v: ScrollBarArgs,
    scrollbar_args_h: ScrollBarArgs,
    child: impl FnOnce() + Send + Sync + 'static,
) {
    scrollable_inner(
        args.clone(),
        state.inner.clone(),
        state.scrollbar_state_v.clone(),
        state.scrollbar_state_h.clone(),
        child,
    );

    if args.vertical {
        scrollbar_v(scrollbar_args_v, state.scrollbar_state_v.clone());
    }

    if args.horizontal {
        scrollbar_h(scrollbar_args_h, state.scrollbar_state_h.clone());
    }

    measure(Box::new(move |input| {
        // Record the final size
        let mut final_size = ComputedData::ZERO;
        // Merge arg constraints with parent constraints
        let self_constraint = Constraint {
            width: args.width,
            height: args.height,
        };
        let mut content_contraint = self_constraint.merge(input.parent_constraint);
        // measure the scrollbar
        if args.vertical {
            let scrollbar_node_id = input.children_ids[1];
            let size = input.measure_child(scrollbar_node_id, input.parent_constraint)?;
            // substract the scrollbar size from the content constraint
            content_contraint.width -= size.width;
            // update the size
            final_size.width += size.width;
        }
        if args.horizontal {
            let scrollbar_node_id = if args.vertical {
                input.children_ids[2]
            } else {
                input.children_ids[1]
            };
            let size = input.measure_child(scrollbar_node_id, input.parent_constraint)?;
            content_contraint.height -= size.height;
            // update the size
            final_size.height += size.height;
        }
        // Measure the content
        let content_node_id = input.children_ids[0];
        let content_measurement = input.measure_child(content_node_id, &content_contraint)?;
        // update the size
        final_size.width += content_measurement.width;
        final_size.height += content_measurement.height;
        // Place childrens
        // place the content at [0, 0]
        input.place_child(content_node_id, PxPosition::ZERO);
        // place the scrollbar at the end
        if args.vertical {
            input.place_child(
                input.children_ids[1],
                PxPosition::new(content_measurement.width, Px::ZERO),
            );
        }
        if args.horizontal {
            let scrollbar_node_id = if args.vertical {
                input.children_ids[2]
            } else {
                input.children_ids[1]
            };
            input.place_child(
                scrollbar_node_id,
                PxPosition::new(Px::ZERO, content_measurement.height),
            );
        }
        // Return the computed data
        Ok(final_size)
    }));
}

#[tessera]
fn scrollable_with_overlay_scrollbar(
    state: Arc<ScrollableState>,
    args: ScrollableArgs,
    scrollbar_args_v: ScrollBarArgs,
    scrollbar_args_h: ScrollBarArgs,
    child: impl FnOnce() + Send + Sync + 'static,
) {
    boxed(
        BoxedArgsBuilder::default()
            .width(args.width)
            .height(args.height)
            .alignment(Alignment::BottomEnd)
            .build()
            .unwrap(),
        |scope| {
            scope.child({
                let state = state.clone();
                let args = args.clone();
                move || {
                    scrollable_inner(
                        args,
                        state.inner.clone(),
                        state.scrollbar_state_v.clone(),
                        state.scrollbar_state_h.clone(),
                        child,
                    );
                }
            });
            scope.child({
                let scrollbar_args_v = scrollbar_args_v.clone();
                let args = args.clone();
                let state = state.clone();
                move || {
                    if args.vertical {
                        scrollbar_v(scrollbar_args_v, state.scrollbar_state_v.clone());
                    }
                }
            });
            scope.child({
                let scrollbar_args_h = scrollbar_args_h.clone();
                let args = args.clone();
                let state = state.clone();
                move || {
                    if args.horizontal {
                        scrollbar_h(scrollbar_args_h, state.scrollbar_state_h.clone());
                    }
                }
            });
        },
    );
}

// Helpers to resolve DimensionValue into concrete Px sizes.
// This reduces duplication in the measurement code and lowers cyclomatic complexity.
fn clamp_wrap(min: Option<Px>, max: Option<Px>, measure: Px) -> Px {
    min.unwrap_or(Px(0))
        .max(measure)
        .min(max.unwrap_or(Px::MAX))
}

fn fill_value(min: Option<Px>, max: Option<Px>, measure: Px) -> Px {
    max.expect("Seems that you are trying to fill an infinite dimension, which is not allowed")
        .max(measure)
        .max(min.unwrap_or(Px(0)))
}

fn resolve_dimension(dim: DimensionValue, measure: Px) -> Px {
    match dim {
        DimensionValue::Fixed(v) => v,
        DimensionValue::Wrap { min, max } => clamp_wrap(min, max, measure),
        DimensionValue::Fill { min, max } => fill_value(min, max, measure),
    }
}

#[tessera]
fn scrollable_inner(
    args: impl Into<ScrollableArgs>,
    state: Arc<RwLock<ScrollableStateInner>>,
    scrollbar_state_v: Arc<RwLock<ScrollBarState>>,
    scrollbar_state_h: Arc<RwLock<ScrollBarState>>,
    child: impl FnOnce(),
) {
    let args: ScrollableArgs = args.into();
    {
        let state = state.clone();
        measure(Box::new(move |input| {
            // Enable clip
            input.enable_clipping();
            // Merge constraints with parent constraints
            let arg_constraint = Constraint {
                width: args.width,
                height: args.height,
            };
            let merged_constraint = input.parent_constraint.merge(&arg_constraint);
            // Now calculate the constraints to child
            let mut child_constraint = merged_constraint;
            // If vertical scrollable, set height to wrap
            if args.vertical {
                child_constraint.height = tessera_ui::DimensionValue::Wrap {
                    min: None,
                    max: None,
                };
            }
            // If horizontal scrollable, set width to wrap
            if args.horizontal {
                child_constraint.width = tessera_ui::DimensionValue::Wrap {
                    min: None,
                    max: None,
                };
            }
            // Measure the child with child constraint
            let child_node_id = input.children_ids[0]; // Scrollable should have exactly one child
            let child_measurement = input.measure_child(child_node_id, &child_constraint)?;
            // Update the child position and size in the state
            state.write().child_size = child_measurement;

            // Update scroll position based on time and get current position for rendering
            let current_child_position = {
                let mut state_guard = state.write();
                state_guard.update_scroll_position(args.scroll_smoothing);
                state_guard.child_position
            };

            // Place child at current interpolated position
            input.place_child(child_node_id, current_child_position);

            // Calculate the size of the scrollable area using helpers to reduce inline branching
            let width = resolve_dimension(merged_constraint.width, child_measurement.width);
            let height = resolve_dimension(merged_constraint.height, child_measurement.height);

            // Pack the size into ComputedData
            let computed_data = ComputedData { width, height };
            // Update the visible size in the state
            state.write().visible_size = computed_data;
            // Return the size of the scrollable area
            Ok(computed_data)
        }));
    }

    // Handle scroll input and position updates
    input_handler(Box::new(move |input| {
        let size = input.computed_data;
        let cursor_pos_option = input.cursor_position_rel;
        let is_cursor_in_component = cursor_pos_option
            .map(|pos| is_position_in_component(size, pos))
            .unwrap_or(false);

        if is_cursor_in_component {
            // Handle scroll events
            for event in input
                .cursor_events
                .iter()
                .filter_map(|event| match &event.content {
                    CursorEventContent::Scroll(event) => Some(event),
                    _ => None,
                })
            {
                let mut state_guard = state.write();

                // Use scroll delta directly (speed already handled in cursor.rs)
                let scroll_delta_x = event.delta_x;
                let scroll_delta_y = event.delta_y;

                // Calculate new target position using saturating arithmetic
                let current_target = state_guard.target_position;
                let new_target = current_target.saturating_offset(
                    Px::saturating_from_f32(scroll_delta_x),
                    Px::saturating_from_f32(scroll_delta_y),
                );

                // Apply bounds constraints immediately before setting target
                let child_size = state_guard.child_size;
                let constrained_target = constrain_position(
                    new_target,
                    &child_size,
                    &input.computed_data,
                    args.vertical,
                    args.horizontal,
                );

                // Set constrained target position
                state_guard.set_target_position(constrained_target);

                // Update scroll activity for AutoHide behavior
                if matches!(args.scrollbar_behavior, ScrollBarBehavior::AutoHide) {
                    // Update vertical scrollbar state if vertical scrolling is enabled
                    if args.vertical {
                        let mut scrollbar_state = scrollbar_state_v.write();
                        scrollbar_state.last_scroll_activity = Some(std::time::Instant::now());
                        scrollbar_state.should_be_visible = true;
                    }
                    // Update horizontal scrollbar state if horizontal scrolling is enabled
                    if args.horizontal {
                        let mut scrollbar_state = scrollbar_state_h.write();
                        scrollbar_state.last_scroll_activity = Some(std::time::Instant::now());
                        scrollbar_state.should_be_visible = true;
                    }
                }
            }

            // Apply bound constraints to the child position
            // To make sure we constrain the target position at least once per frame
            let target = state.read().target_position;
            let child_size = state.read().child_size;
            let constrained_position = constrain_position(
                target,
                &child_size,
                &input.computed_data,
                args.vertical,
                args.horizontal,
            );
            state.write().set_target_position(constrained_position);

            // Block cursor events to prevent propagation
            input.cursor_events.clear();
        }

        // Update scroll position based on time (only once per frame, after handling events)
        state.write().update_scroll_position(args.scroll_smoothing);
    }));

    // Add child component
    child();
}

/// Constrains a position to stay within the scrollable bounds.
///
/// Split per-axis logic into a helper to simplify reasoning and reduce cyclomatic complexity.
fn constrain_axis(pos: Px, child_len: Px, container_len: Px) -> Px {
    if pos > Px::ZERO {
        Px::ZERO
    } else if pos.saturating_add(child_len) < container_len {
        container_len.saturating_sub(child_len)
    } else {
        pos
    }
}

fn constrain_position(
    position: PxPosition,
    child_size: &ComputedData,
    container_size: &ComputedData,
    vertical_scrollable: bool,
    horizontal_scrollable: bool,
) -> PxPosition {
    let x = if horizontal_scrollable {
        constrain_axis(position.x, child_size.width, container_size.width)
    } else {
        Px::ZERO
    };

    let y = if vertical_scrollable {
        constrain_axis(position.y, child_size.height, container_size.height)
    } else {
        Px::ZERO
    };

    PxPosition { x, y }
}
