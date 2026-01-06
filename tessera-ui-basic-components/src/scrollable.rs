//! A container that allows its content to be scrolled.
//!
//! ## Usage
//!
//! Use to display content that might overflow the available space.
mod scrollbar;
use std::time::Instant;

use derive_setters::Setters;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, MeasurementError,
    Modifier, Px, PxPosition, State,
    layout::{LayoutInput, LayoutOutput, LayoutSpec, RenderInput},
    remember, tessera,
};

use crate::{
    alignment::Alignment,
    boxed::{BoxedArgs, boxed},
    modifier::ModifierExt,
    pos_misc::is_position_in_component,
    scrollable::scrollbar::{ScrollBarArgs, ScrollBarState, scrollbar_h, scrollbar_v},
};

/// Arguments for the `scrollable` container.
#[derive(Debug, Setters, Clone)]
pub struct ScrollableArgs {
    /// Modifier chain applied to the scrollable subtree.
    pub modifier: Modifier,
    /// Is vertical scrollable?
    /// Defaults to `true` since most scrollable areas are vertical.
    pub vertical: bool,
    /// Is horizontal scrollable?
    /// Defaults to `false` since most scrollable areas are not horizontal.
    pub horizontal: bool,
    /// Scroll smoothing factor (0.0 = instant, 1.0 = very smooth).
    /// Defaults to 0.05 for very responsive but still smooth scrolling.
    pub scroll_smoothing: f32,
    /// The behavior of the scrollbar visibility.
    pub scrollbar_behavior: ScrollBarBehavior,
    /// The color of the scrollbar track.
    pub scrollbar_track_color: Color,
    /// The color of the scrollbar thumb.
    pub scrollbar_thumb_color: Color,
    /// The color of the scrollbar thumb when hovered.
    pub scrollbar_thumb_hover_color: Color,
    /// The layout of the scrollbar relative to the content.
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
    /// The scrollbar is placed alongside the content (takes up space in the
    /// layout).
    Alongside,
    /// The scrollbar is overlaid on top of the content (doesn't take up space).
    Overlay,
}

impl Default for ScrollableArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new().fill_max_size(),
            vertical: true,
            horizontal: false,
            scroll_smoothing: 0.05,
            scrollbar_behavior: ScrollBarBehavior::AlwaysVisible,
            scrollbar_track_color: Color::new(0.0, 0.0, 0.0, 0.1),
            scrollbar_thumb_color: Color::new(0.0, 0.0, 0.0, 0.3),
            scrollbar_thumb_hover_color: Color::new(0.0, 0.0, 0.0, 0.5),
            scrollbar_layout: ScrollBarLayout::Alongside,
        }
    }
}

/// Holds the state for a `scrollable` component, managing scroll position and
/// interaction.
///
/// It tracks the current and target scroll positions, the size of the
/// scrollable content, and focus state.
///
/// The scroll position is smoothly interpolated over time to create a fluid
/// scrolling effect.
#[derive(Clone)]
pub struct ScrollableController {
    /// The current position of the child component (for rendering)
    child_position: PxPosition,
    /// The target position of the child component (scrolling destination)
    target_position: PxPosition,
    /// The child component size
    child_size: ComputedData,
    /// The visible area size
    visible_size: ComputedData,
    /// Optional override for the child size used to clamp scroll extents.
    override_child_size: Option<ComputedData>,
    /// Last frame time for delta time calculation
    last_frame_time: Option<Instant>,
    /// The state for vertical scrollbar
    scrollbar_state_v: ScrollBarState,
    /// The state for horizontal scrollbar
    scrollbar_state_h: ScrollBarState,
}

impl Default for ScrollableController {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollableController {
    /// Creates a new `ScrollableController` with default values.
    pub fn new() -> Self {
        Self {
            child_position: PxPosition::ZERO,
            target_position: PxPosition::ZERO,
            child_size: ComputedData::ZERO,
            visible_size: ComputedData::ZERO,
            override_child_size: None,
            last_frame_time: None,
            scrollbar_state_v: ScrollBarState::default(),
            scrollbar_state_h: ScrollBarState::default(),
        }
    }

    /// Returns the current child position relative to the scrollable container.
    ///
    /// This is primarily useful for components that need to implement custom
    /// virtualization strategies (e.g. lazy lists) and must know the current
    /// scroll offset. Values are clamped by the scroll logic, so consumers
    /// can safely derive their offset from the returned position.
    pub fn child_position(&self) -> PxPosition {
        self.child_position
    }

    /// Returns the currently visible viewport size of the scrollable container.
    pub fn visible_size(&self) -> ComputedData {
        self.visible_size
    }

    fn child_size(&self) -> ComputedData {
        self.child_size
    }

    /// Overrides the child size used for scroll extent calculation.
    pub fn override_child_size(&mut self, size: ComputedData) {
        self.override_child_size = Some(size);
    }

    fn target_position(&self) -> PxPosition {
        self.target_position
    }

    fn set_target_position(&mut self, target: PxPosition) {
        self.target_position = target;
    }

    /// Instantly sets the scroll position without animation.
    ///
    /// This is useful for restoring a saved scroll position when remounting
    /// a component.
    pub fn set_scroll_position(&mut self, position: PxPosition) {
        self.child_position = position;
        self.target_position = position;
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

    pub(crate) fn scrollbar_state_v(&self) -> ScrollBarState {
        self.scrollbar_state_v.clone()
    }

    pub(crate) fn scrollbar_state_h(&self) -> ScrollBarState {
        self.scrollbar_state_h.clone()
    }
}

#[derive(Clone, PartialEq)]
struct ScrollableAlongsideLayout {
    vertical: bool,
    horizontal: bool,
}

impl LayoutSpec for ScrollableAlongsideLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let mut final_size = ComputedData::ZERO;
        let mut content_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );

        if self.vertical {
            let scrollbar_node_id = input.children_ids()[1];
            let size = input.measure_child_in_parent_constraint(scrollbar_node_id)?;
            content_constraint.width -= size.width;
            final_size.width += size.width;
        }

        if self.horizontal {
            let scrollbar_node_id = if self.vertical {
                input.children_ids()[2]
            } else {
                input.children_ids()[1]
            };
            let size = input.measure_child_in_parent_constraint(scrollbar_node_id)?;
            content_constraint.height -= size.height;
            final_size.height += size.height;
        }

        let content_node_id = input.children_ids()[0];
        let content_measurement = input.measure_child(content_node_id, &content_constraint)?;
        final_size.width += content_measurement.width;
        final_size.height += content_measurement.height;

        output.place_child(content_node_id, PxPosition::ZERO);
        if self.vertical {
            output.place_child(
                input.children_ids()[1],
                PxPosition::new(content_measurement.width, Px::ZERO),
            );
        }
        if self.horizontal {
            let scrollbar_node_id = if self.vertical {
                input.children_ids()[2]
            } else {
                input.children_ids()[1]
            };
            output.place_child(
                scrollbar_node_id,
                PxPosition::new(Px::ZERO, content_measurement.height),
            );
        }

        Ok(final_size)
    }
}

#[derive(Clone)]
struct ScrollableInnerLayout {
    controller: State<ScrollableController>,
    vertical: bool,
    horizontal: bool,
    child_position: PxPosition,
    has_override: bool,
}

impl PartialEq for ScrollableInnerLayout {
    fn eq(&self, other: &Self) -> bool {
        self.vertical == other.vertical
            && self.horizontal == other.horizontal
            && self.child_position == other.child_position
            && self.has_override == other.has_override
    }
}

impl LayoutSpec for ScrollableInnerLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let merged_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );
        let mut child_constraint = merged_constraint;

        if self.vertical {
            child_constraint.height = DimensionValue::Wrap {
                min: None,
                max: None,
            };
        }
        if self.horizontal {
            child_constraint.width = DimensionValue::Wrap {
                min: None,
                max: None,
            };
        }

        let child_node_id = input.children_ids()[0];
        let child_measurement = input.measure_child(child_node_id, &child_constraint)?;
        let current_child_position = self.child_position;
        self.controller.with_mut(|c| {
            if let Some(override_size) = c.override_child_size.take() {
                c.child_size = override_size;
            } else {
                c.child_size = child_measurement;
            }
        });

        output.place_child(child_node_id, current_child_position);

        let mut width = resolve_dimension(merged_constraint.width, child_measurement.width);
        let mut height = resolve_dimension(merged_constraint.height, child_measurement.height);

        if let Some(parent_max_width) = input.parent_constraint().width().get_max() {
            width = width.min(parent_max_width);
        }
        if let Some(parent_max_height) = input.parent_constraint().height().get_max() {
            height = height.min(parent_max_height);
        }

        let computed_data = ComputedData { width, height };
        self.controller.with_mut(|c| c.visible_size = computed_data);
        Ok(computed_data)
    }

    fn record(&self, input: &RenderInput<'_>) {
        input.metadata_mut().clips_children = true;
    }
}

/// # scrollable
///
/// Creates a container that makes its content scrollable when it overflows.
///
/// ## Usage
///
/// Wrap a large component or a long list of items to allow the user to scroll
/// through them.
///
/// ## Parameters
///
/// - `args` — configures the scrollable area's dimensions, direction, and
///   scrollbar appearance; see [`ScrollableArgs`].
/// - `child` — a closure that renders the content to be scrolled.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{Dp, Modifier, tessera};
/// use tessera_ui_basic_components::{
///     column::{ColumnArgs, column},
///     modifier::ModifierExt as _,
///     scrollable::{ScrollableArgs, scrollable},
///     text::{TextArgs, text},
/// };
///
/// #[tessera]
/// fn demo() {
///     scrollable(
///         ScrollableArgs {
///             modifier: Modifier::new().height(Dp(100.0)),
///             ..Default::default()
///         },
///         || {
///             column(ColumnArgs::default(), |scope| {
///                 for i in 0..20 {
///                     let text_content = format!("Item #{}", i + 1);
///                     scope.child(|| {
///                         text(TextArgs::default().text(text_content));
///                     });
///                 }
///             });
///         },
///     );
/// }
/// ```
#[tessera]
pub fn scrollable(args: impl Into<ScrollableArgs>, child: impl FnOnce() + Send + Sync + 'static) {
    let controller = remember(ScrollableController::new);
    scrollable_with_controller(args, controller, child);
}

/// # scrollable_with_controller
///
/// Scrollable container variant that accepts an explicit controller.
///
/// ## Usage
///
/// Use when you need to observe or drive scroll position externally (e.g.,
/// synchronize multiple scroll areas).
///
/// ## Parameters
///
/// - `args` — configures the scrollable area's dimensions, direction, and
///   scrollbar appearance; see [`ScrollableArgs`].
/// - `controller` — a [`ScrollableController`] that stores the scroll offsets
///   and viewport data.
/// - `child` — a closure that renders the content to be scrolled.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{Dp, Modifier, remember, tessera};
/// use tessera_ui_basic_components::{
///     column::{ColumnArgs, column},
///     modifier::ModifierExt as _,
///     scrollable::{ScrollableArgs, ScrollableController, scrollable_with_controller},
///     text::{TextArgs, text},
/// };
///
/// #[tessera]
/// fn demo() {
///     let controller = remember(ScrollableController::new);
///     scrollable_with_controller(
///         ScrollableArgs {
///             modifier: Modifier::new().height(Dp(120.0)),
///             ..Default::default()
///         },
///         controller,
///         || {
///             column(ColumnArgs::default(), |scope| {
///                 for i in 0..10 {
///                     let text_content = format!("Row #{i}");
///                     scope.child(|| {
///                         text(TextArgs::default().text(text_content));
///                     });
///                 }
///             });
///         },
///     );
/// }
/// ```
#[tessera]
pub fn scrollable_with_controller(
    args: impl Into<ScrollableArgs>,
    controller: State<ScrollableController>,
    child: impl FnOnce() + Send + Sync + 'static,
) {
    let args: ScrollableArgs = args.into();
    let modifier = args.modifier;

    // Create separate ScrollBarArgs for vertical and horizontal scrollbars
    let scrollbar_args_v = ScrollBarArgs {
        total: controller.with(|c| c.child_size().height),
        visible: controller.with(|c| c.visible_size().height),
        offset: controller.with(|c| c.child_position().y),
        thickness: Dp(8.0), // Default scrollbar thickness
        state: controller,
        scrollbar_behavior: args.scrollbar_behavior.clone(),
        track_color: args.scrollbar_track_color,
        thumb_color: args.scrollbar_thumb_color,
        thumb_hover_color: args.scrollbar_thumb_hover_color,
    };

    let scrollbar_args_h = ScrollBarArgs {
        total: controller.with(|c| c.child_size().width),
        visible: controller.with(|c| c.visible_size().width),
        offset: controller.with(|c| c.child_position().x),
        thickness: Dp(8.0), // Default scrollbar thickness
        state: controller,
        scrollbar_behavior: args.scrollbar_behavior.clone(),
        track_color: args.scrollbar_track_color,
        thumb_color: args.scrollbar_thumb_color,
        thumb_hover_color: args.scrollbar_thumb_hover_color,
    };

    match args.scrollbar_layout {
        ScrollBarLayout::Alongside => {
            modifier.run(move || {
                scrollable_with_alongside_scrollbar(
                    controller,
                    args,
                    scrollbar_args_v,
                    scrollbar_args_h,
                    child,
                );
            });
        }
        ScrollBarLayout::Overlay => {
            modifier.run(move || {
                scrollable_with_overlay_scrollbar(
                    controller,
                    args,
                    scrollbar_args_v,
                    scrollbar_args_h,
                    child,
                );
            });
        }
    }
}

#[tessera]
fn scrollable_with_alongside_scrollbar(
    controller: State<ScrollableController>,
    args: ScrollableArgs,
    scrollbar_args_v: ScrollBarArgs,
    scrollbar_args_h: ScrollBarArgs,
    child: impl FnOnce() + Send + Sync + 'static,
) {
    let scrollbar_v_state = controller.with(|c| c.scrollbar_state_v());
    let scrollbar_h_state = controller.with(|c| c.scrollbar_state_h());

    scrollable_inner(
        args.clone(),
        controller,
        scrollbar_v_state.clone(),
        scrollbar_h_state.clone(),
        child,
    );

    if args.vertical {
        scrollbar_v(scrollbar_args_v, scrollbar_v_state);
    }

    if args.horizontal {
        scrollbar_h(scrollbar_args_h, scrollbar_h_state);
    }

    layout(ScrollableAlongsideLayout {
        vertical: args.vertical,
        horizontal: args.horizontal,
    });
}

#[tessera]
fn scrollable_with_overlay_scrollbar(
    controller: State<ScrollableController>,
    args: ScrollableArgs,
    scrollbar_args_v: ScrollBarArgs,
    scrollbar_args_h: ScrollBarArgs,
    child: impl FnOnce() + Send + Sync + 'static,
) {
    boxed(
        BoxedArgs::default()
            .modifier(Modifier::new().fill_max_size())
            .alignment(Alignment::BottomEnd),
        |scope| {
            scope.child({
                let args = args.clone();
                let scrollbar_v_state = controller.with(|c| c.scrollbar_state_v());
                let scrollbar_h_state = controller.with(|c| c.scrollbar_state_h());
                move || {
                    scrollable_inner(
                        args,
                        controller,
                        scrollbar_v_state,
                        scrollbar_h_state,
                        child,
                    );
                }
            });
            scope.child({
                let scrollbar_args_v = scrollbar_args_v.clone();
                let args = args.clone();
                let scrollbar_v_state = controller.with(|c| c.scrollbar_state_v());
                move || {
                    if args.vertical {
                        scrollbar_v(scrollbar_args_v, scrollbar_v_state);
                    }
                }
            });
            scope.child({
                let scrollbar_args_h = scrollbar_args_h.clone();
                let args = args.clone();
                let scrollbar_h_state = controller.with(|c| c.scrollbar_state_h());
                move || {
                    if args.horizontal {
                        scrollbar_h(scrollbar_args_h, scrollbar_h_state);
                    }
                }
            });
        },
    );
}

// Helpers to resolve DimensionValue into concrete Px sizes.
// This reduces duplication in the measurement code and lowers cyclomatic
// complexity.
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
    args: ScrollableArgs,
    controller: State<ScrollableController>,
    scrollbar_state_v: ScrollBarState,
    scrollbar_state_h: ScrollBarState,
    child: impl FnOnce(),
) {
    controller.with_mut(|c| c.update_scroll_position(args.scroll_smoothing));
    let child_position = controller.with(|c| c.child_position());
    let has_override = controller.with(|c| c.override_child_size.is_some());
    layout(ScrollableInnerLayout {
        controller,
        vertical: args.vertical,
        horizontal: args.horizontal,
        child_position,
        has_override,
    });

    // Handle scroll input and position updates
    input_handler(move |input| {
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
                controller.with_mut(|c| {
                    // Use scroll delta directly (speed already handled in cursor.rs)
                    let scroll_delta_x = event.delta_x;
                    let scroll_delta_y = event.delta_y;

                    // Calculate new target position using saturating arithmetic
                    let current_target = c.target_position;
                    let new_target = current_target.saturating_offset(
                        Px::saturating_from_f32(scroll_delta_x),
                        Px::saturating_from_f32(scroll_delta_y),
                    );

                    // Apply bounds constraints immediately before setting target
                    let child_size = c.child_size;
                    let constrained_target = constrain_position(
                        new_target,
                        &child_size,
                        &input.computed_data,
                        args.vertical,
                        args.horizontal,
                    );

                    // Set constrained target position
                    c.set_target_position(constrained_target);
                });

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
            let target = controller.with(|c| c.target_position());
            let child_size = controller.with(|c| c.child_size());
            let constrained_position = constrain_position(
                target,
                &child_size,
                &input.computed_data,
                args.vertical,
                args.horizontal,
            );
            controller.with_mut(|c| c.set_target_position(constrained_position));

            // Block cursor events to prevent propagation
            input.cursor_events.clear();
        }
    });

    // Add child component
    child();
}

/// Constrains a position to stay within the scrollable bounds.
///
/// Split per-axis logic into a helper to simplify reasoning and reduce
/// cyclomatic complexity.
fn constrain_axis(pos: Px, child_len: Px, container_len: Px) -> Px {
    if child_len <= container_len {
        return Px::ZERO;
    }

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
