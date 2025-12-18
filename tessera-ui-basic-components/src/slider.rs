//! An interactive slider component for selecting a value in a range.
//!
//! ## Usage
//!
//! Use to allow users to select a value from a continuous range.
use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, MeasureInput, MeasurementError, Modifier,
    Px, PxPosition, State,
    accessibility::AccessibilityNode,
    accesskit::{Action, Role},
    focus_state::Focus,
    remember, tessera, use_context,
};

use crate::{
    pipelines::image_vector::command::VectorTintMode,
    theme::{MaterialAlpha, MaterialTheme},
};

use interaction::{
    apply_range_slider_accessibility, apply_slider_accessibility, handle_range_slider_state,
    handle_slider_state, snap_fraction,
};
use layout::{
    CenteredSliderLayout, RangeSliderLayout, SliderLayout, fallback_component_width,
    range_slider_layout, resolve_component_width, slider_layout_with_handle_width,
};
use render::{
    render_active_segment, render_centered_stops, render_centered_tracks, render_handle,
    render_inactive_segment, render_range_stops, render_range_tracks, render_stop_indicator,
    render_tick,
};

pub use interaction::RangeSliderController;

mod interaction;
mod layout;
mod render;

const ACCESSIBILITY_STEP: f32 = 0.05;
const MIN_TOUCH_TARGET: Dp = Dp(40.0);
const HANDLE_GAP: Dp = Dp(6.0);
const STOP_INDICATOR_DIAMETER: Dp = Dp(4.0);

fn tick_fractions(steps: usize) -> Vec<f32> {
    if steps == 0 {
        return Vec::new();
    }
    let denom = steps as f32 + 1.0;
    (0..=steps + 1).map(|i| i as f32 / denom).collect()
}

struct RangeThumbAccessibilityArgs<'a> {
    key: &'static str,
    label: Option<&'a String>,
    description: Option<&'a String>,
    fallback_description: &'static str,
    steps: usize,
    disabled: bool,
    value: f32,
    min: f32,
    max: f32,
    on_change: Arc<dyn Fn(f32) + Send + Sync>,
}

fn set_range_thumb_accessibility(
    input: &MeasureInput,
    thumb_id: tessera_ui::NodeId,
    args: RangeThumbAccessibilityArgs<'_>,
) {
    let mut node = AccessibilityNode::new()
        .with_role(Role::Slider)
        .with_numeric_value(args.value as f64)
        .with_numeric_range(args.min as f64, args.max as f64)
        .focusable()
        .with_key(args.key);

    if let Some(label) = args.label {
        node = node.with_label(label.clone());
    }

    let description = args
        .description
        .map(|d| format!("{d} ({})", args.fallback_description))
        .unwrap_or_else(|| args.fallback_description.to_string());
    node = node.with_description(description);

    if args.disabled {
        node.disabled = true;
    } else {
        node.actions = vec![Action::Increment, Action::Decrement];
    }

    if let Some(mut metadata) = input.metadatas.get_mut(&thumb_id) {
        metadata.accessibility = Some(node);
        metadata.accessibility_action_handler = if args.disabled {
            None
        } else {
            let delta = if args.steps == 0 {
                ACCESSIBILITY_STEP
            } else {
                1.0 / (args.steps as f32 + 1.0)
            };
            let value = args.value;
            let min = args.min;
            let max = args.max;
            let steps = args.steps;
            let on_change = args.on_change.clone();
            Some(Box::new(move |action| {
                let next = match action {
                    Action::Increment => value + delta,
                    Action::Decrement => value - delta,
                    _ => return,
                };
                let next = snap_fraction(next, steps).clamp(min, max);
                on_change(next);
            }))
        };
    }
}

struct RangeSliderMeasureArgs {
    start: f32,
    end: f32,
    start_handle_width: Px,
    end_handle_width: Px,
    steps: usize,
    disabled: bool,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    on_change: Arc<dyn Fn((f32, f32)) + Send + Sync>,
}

/// Controller for the `slider` component.
pub struct SliderController {
    is_dragging: bool,
    focus: Focus,
    is_hovered: bool,
}

impl SliderController {
    /// Creates a new slider controller.
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            focus: Focus::new(),
            is_hovered: false,
        }
    }

    /// Returns whether the slider handle is currently being dragged.
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Manually sets the dragging flag. Useful for custom gesture integrations.
    pub fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
    }

    /// Requests focus for the slider.
    pub fn request_focus(&mut self) {
        self.focus.request_focus();
    }

    /// Clears focus from the slider if it is currently focused.
    pub fn clear_focus(&mut self) {
        self.focus.unfocus();
    }

    /// Returns `true` if this slider currently holds focus.
    pub fn is_focused(&self) -> bool {
        self.focus.is_focused()
    }

    /// Returns `true` if the cursor is hovering over this slider.
    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }
}

impl Default for SliderController {
    fn default() -> Self {
        Self::new()
    }
}

/// Size variants for the slider component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SliderSize {
    /// Extra Small (default).
    #[default]
    ExtraSmall,
    /// Small.
    Small,
    /// Medium.
    Medium,
    /// Large.
    Large,
    /// Extra Large.
    ExtraLarge,
}

/// Arguments for the `slider` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct SliderArgs {
    /// Modifier chain applied to the slider subtree.
    #[builder(default = "Modifier::new()")]
    pub modifier: Modifier,
    /// The current value of the slider, ranging from 0.0 to 1.0.
    #[builder(default = "0.0")]
    pub value: f32,
    /// Callback function triggered when the slider's value changes.
    #[builder(default = "Arc::new(|_| {})")]
    pub on_change: Arc<dyn Fn(f32) + Send + Sync>,
    /// Size variant of the slider.
    #[builder(default)]
    pub size: SliderSize,
    /// The color of the active part of the track (progress fill).
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.primary")]
    pub active_track_color: Color,
    /// The color of the inactive part of the track (background).
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.secondary_container")]
    pub inactive_track_color: Color,
    /// The thickness of the handle indicator.
    #[builder(default = "Dp(4.0)")]
    pub thumb_diameter: Dp,
    /// Color of the handle indicator.
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.primary")]
    pub thumb_color: Color,
    /// Disable interaction.
    #[builder(default = "false")]
    pub disabled: bool,
    /// Optional accessibility label read by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
    /// Whether to show the stop indicators at the ends of the track.
    #[builder(default = "true")]
    pub show_stop_indicator: bool,
    /// Number of discrete steps between 0.0 and 1.0.
    ///
    /// When set to a value greater than 0, the slider value snaps to
    /// `steps + 2` evenly spaced tick positions (including both ends).
    #[builder(default = "0")]
    pub steps: usize,
    /// Optional icon content to display at the start of the slider (only for
    /// Medium sizes and above).
    #[builder(default, setter(strip_option, into))]
    pub inset_icon: Option<crate::icon::IconContent>,
}

/// Arguments for the `range_slider` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct RangeSliderArgs {
    /// Modifier chain applied to the range slider subtree.
    #[builder(default = "Modifier::new()")]
    pub modifier: Modifier,
    /// The current range values (start, end), each between 0.0 and 1.0.
    #[builder(default = "(0.0, 1.0)")]
    pub value: (f32, f32),

    /// Callback function triggered when the range values change.
    #[builder(default = "Arc::new(|_| {})")]
    pub on_change: Arc<dyn Fn((f32, f32)) + Send + Sync>,

    /// Size variant of the slider.
    #[builder(default)]
    pub size: SliderSize,

    /// The color of the active part of the track (range fill).
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.primary")]
    pub active_track_color: Color,

    /// The color of the inactive part of the track (background).
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.secondary_container")]
    pub inactive_track_color: Color,

    /// The thickness of the handle indicators.
    #[builder(default = "Dp(4.0)")]
    pub thumb_diameter: Dp,

    /// Color of the handle indicators.
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.primary")]
    pub thumb_color: Color,

    /// Disable interaction.
    #[builder(default = "false")]
    pub disabled: bool,
    /// Optional accessibility label.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,

    /// Whether to show the stop indicators at the ends of the track.
    #[builder(default = "true")]
    pub show_stop_indicator: bool,
    /// Number of discrete steps between 0.0 and 1.0.
    ///
    /// When set to a value greater than 0, the slider values snap to
    /// `steps + 2` evenly spaced tick positions (including both ends).
    #[builder(default = "0")]
    pub steps: usize,
}

fn measure_slider(
    input: &MeasureInput,
    layout: SliderLayout,
    clamped_value: f32,
    has_inset_icon: bool,
    handle_width: Px,
    steps: usize,
) -> Result<ComputedData, MeasurementError> {
    let self_width = layout.component_width;
    let self_height = layout.component_height;

    let active_id = input.children_ids[0];
    let inactive_id = input.children_ids[1];

    // Order in render: active, inactive, [icon], [ticks], [stop], handle
    let mut current_index = 2;

    let icon_id = if has_inset_icon {
        let id = input.children_ids.get(current_index).copied();
        current_index += 1;
        id
    } else {
        None
    };

    let tick_count = if steps == 0 { 0 } else { steps + 2 };
    let tick_ids = &input.children_ids[current_index..current_index + tick_count];
    current_index += tick_count;

    let stop_id = if layout.show_stop_indicator {
        let id = input.children_ids.get(current_index).copied();
        current_index += 1;
        id
    } else {
        None
    };

    let handle_id = input.children_ids[current_index];

    let active_width = layout.active_width(clamped_value);
    let inactive_width = layout.inactive_width(clamped_value);

    let active_constraint = Constraint::new(
        DimensionValue::Fixed(active_width),
        DimensionValue::Fixed(layout.track_height),
    );
    input.measure_child(active_id, &active_constraint)?;
    input.place_child(active_id, PxPosition::new(Px(0), layout.track_y));

    let inactive_constraint = Constraint::new(
        DimensionValue::Fixed(inactive_width),
        DimensionValue::Fixed(layout.track_height),
    );
    input.measure_child(inactive_id, &inactive_constraint)?;
    input.place_child(
        inactive_id,
        PxPosition::new(
            Px(active_width.0 + layout.handle_gap.0 * 2 + handle_width.0),
            layout.track_y,
        ),
    );

    let handle_constraint = Constraint::new(
        DimensionValue::Fixed(handle_width),
        DimensionValue::Fixed(layout.handle_height),
    );
    input.measure_child(handle_id, &handle_constraint)?;

    let handle_center = layout.handle_center(clamped_value);
    let handle_offset = layout.center_child_offset(handle_width);
    input.place_child(
        handle_id,
        PxPosition::new(Px(handle_center.x.0 - handle_offset.0), layout.handle_y),
    );

    if let Some(stop_id) = stop_id {
        let stop_size = layout.stop_indicator_diameter;
        let stop_constraint = Constraint::new(
            DimensionValue::Fixed(stop_size),
            DimensionValue::Fixed(stop_size),
        );
        input.measure_child(stop_id, &stop_constraint)?;
        let stop_offset = layout.center_child_offset(layout.stop_indicator_diameter);
        let inactive_start = active_width.0 + layout.handle_gap.0 * 2 + handle_width.0;
        let corner = layout.track_corner_radius.to_px();
        let stop_center_x = Px(inactive_start + inactive_width.0 - corner.0);
        input.place_child(
            stop_id,
            PxPosition::new(Px(stop_center_x.0 - stop_offset.0), layout.stop_indicator_y),
        );
    }

    if let Some(icon_id) = icon_id
        && let Some(icon_size) = layout.icon_size
    {
        let icon_constraint = Constraint::new(
            DimensionValue::Wrap {
                min: None,
                max: Some(icon_size.into()),
            },
            DimensionValue::Wrap {
                min: None,
                max: Some(icon_size.into()),
            },
        );
        let icon_measured = input.measure_child(icon_id, &icon_constraint)?;

        // Icon placement: 8dp padding from left edge, vertically centered within the
        // track
        let icon_padding = Dp(8.0).to_px();
        let icon_y = layout.track_y + Px((layout.track_height.0 - icon_measured.height.0) / 2);
        input.place_child(icon_id, PxPosition::new(icon_padding, icon_y));
    }

    if steps > 0 {
        let tick_size = layout.stop_indicator_diameter;
        let tick_constraint = Constraint::new(
            DimensionValue::Fixed(tick_size),
            DimensionValue::Fixed(tick_size),
        );
        let tick_offset = layout.center_child_offset(tick_size);
        let start_x = layout.handle_gap.to_f32() + handle_width.to_f32() / 2.0;
        for (i, tick_id) in tick_ids.iter().copied().enumerate() {
            input.measure_child(tick_id, &tick_constraint)?;
            let fraction = i as f32 / (steps as f32 + 1.0);
            let tick_center_x = start_x + fraction * layout.track_total_width.to_f32();
            input.place_child(
                tick_id,
                PxPosition::new(
                    Px(tick_center_x.round() as i32 - tick_offset.0),
                    layout.stop_indicator_y,
                ),
            );
        }
    }

    Ok(ComputedData {
        width: self_width,
        height: self_height,
    })
}

#[derive(Clone, Copy)]
struct SliderColors {
    active_track: Color,
    inactive_track: Color,
    thumb: Color,
}

fn slider_colors(args: &SliderArgs) -> SliderColors {
    if args.disabled {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        let disabled_thumb = scheme
            .surface
            .blend_over(scheme.on_surface, MaterialAlpha::DISABLED_CONTENT);
        return SliderColors {
            active_track: scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTENT),
            inactive_track: scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTAINER),
            thumb: disabled_thumb,
        };
    }

    SliderColors {
        active_track: args.active_track_color,
        inactive_track: args.inactive_track_color,
        thumb: args.thumb_color,
    }
}

fn range_slider_colors(args: &RangeSliderArgs) -> SliderColors {
    if args.disabled {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        let disabled_thumb = scheme
            .surface
            .blend_over(scheme.on_surface, MaterialAlpha::DISABLED_CONTENT);
        return SliderColors {
            active_track: scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTENT),
            inactive_track: scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTAINER),
            thumb: disabled_thumb,
        };
    }

    SliderColors {
        active_track: args.active_track_color,
        inactive_track: args.inactive_track_color,
        thumb: args.thumb_color,
    }
}

/// # slider
///
/// Renders an interactive slider with a bar-style handle for selecting a value
/// between 0.0 and 1.0.
///
/// ## Usage
///
/// Use for settings like volume or brightness, or for any user-adjustable
/// value.
///
/// ## Parameters
///
/// - `args` — configures the slider's value, appearance, and callbacks; see
///   [`SliderArgs`].
/// - `controller` — optional; use [`slider_with_controller`] to provide your
///   own controller.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use std::sync::Arc;
/// use tessera_ui::{Dp, Modifier};
/// use tessera_ui_basic_components::modifier::ModifierExt as _;
/// use tessera_ui_basic_components::slider::{SliderArgsBuilder, slider};
///
/// slider(
///     SliderArgsBuilder::default()
///         .modifier(Modifier::new().width(Dp(200.0)))
///         .value(0.5)
///         .on_change(Arc::new(|new_value| {
///             // In a real app, you would update your state here.
///             println!("Slider value changed to: {}", new_value);
///         }))
///         .build()
///         .unwrap(),
/// );
/// # }
/// # component();
/// ```
#[tessera]
pub fn slider(args: impl Into<SliderArgs>) {
    let args: SliderArgs = args.into();
    let controller = remember(SliderController::new);
    slider_with_controller(args, controller);
}

/// # slider_with_controller
///
/// Controlled slider variant.
///
/// ## Usage
///
/// Use when you need to manage the slider's interactive state externally.
///
/// ## Parameters
///
/// - `args` — configures the slider's value, appearance, and callbacks; see
///   [`SliderArgs`].
/// - `controller` — the slider controller to manage interactive state.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use std::sync::Arc;
/// use tessera_ui::{Dp, Modifier, remember};
/// use tessera_ui_basic_components::modifier::ModifierExt as _;
/// use tessera_ui_basic_components::slider::{
///     SliderArgsBuilder, SliderController, slider_with_controller,
/// };
///
/// let controller = remember(|| SliderController::new());
/// slider_with_controller(
///     SliderArgsBuilder::default()
///         .modifier(Modifier::new().width(Dp(200.0)))
///         .value(0.5)
///         .on_change(Arc::new(|new_value| {
///             println!("Slider value changed to: {}", new_value);
///         }))
///         .build()
///         .unwrap(),
///     controller,
/// );
/// # }
/// # component();
/// ```
#[tessera]
pub fn slider_with_controller(args: impl Into<SliderArgs>, controller: State<SliderController>) {
    let args: SliderArgs = args.into();
    let modifier = args.modifier;
    modifier.run(move || slider_with_controller_inner(args, controller));
}

#[tessera]
fn slider_with_controller_inner(args: SliderArgs, controller: State<SliderController>) {
    let initial_width = fallback_component_width(&args);
    let clamped_value = args.value.clamp(0.0, 1.0);
    let (is_dragging, is_focused) = controller.with(|c| (c.is_dragging(), c.is_focused()));
    let base_handle_width = args.thumb_diameter.to_px();
    let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
    let handle_width = if is_dragging || is_focused {
        pressed_handle_width
    } else {
        base_handle_width
    };
    let layout = slider_layout_with_handle_width(&args, initial_width, handle_width);
    let colors = slider_colors(&args);

    render_active_segment(layout, &colors);
    render_inactive_segment(layout, &colors);

    if let Some(icon_size) = layout.icon_size
        && let Some(inset_icon) = args.inset_icon.as_ref()
    {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        let tint = if args.disabled {
            scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTENT)
        } else {
            scheme.on_primary
        };

        crate::icon::icon(
            crate::icon::IconArgsBuilder::default()
                .content(inset_icon.clone())
                .tint(tint)
                .tint_mode(VectorTintMode::Solid)
                .size(icon_size)
                .build()
                .expect("Failed to build icon args"),
        );
    }

    if args.steps > 0 {
        for fraction in tick_fractions(args.steps) {
            let is_active = fraction <= clamped_value;
            let color = if is_active {
                colors.inactive_track
            } else {
                colors.active_track
            };
            render_tick(layout.stop_indicator_diameter, color);
        }
    }
    if layout.show_stop_indicator {
        render_stop_indicator(layout, &colors);
    }
    render_handle(layout, handle_width, &colors);

    let cloned_args = args.clone();
    input_handler(Box::new(move |mut input| {
        let (is_dragging, is_focused) = controller.with(|c| (c.is_dragging(), c.is_focused()));
        let base_handle_width = cloned_args.thumb_diameter.to_px();
        let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
        let handle_width = if is_dragging || is_focused {
            pressed_handle_width
        } else {
            base_handle_width
        };
        let resolved_layout =
            slider_layout_with_handle_width(&cloned_args, input.computed_data.width, handle_width);
        handle_slider_state(&mut input, controller, &cloned_args, &resolved_layout);
        apply_slider_accessibility(
            &mut input,
            &cloned_args,
            clamped_value,
            &cloned_args.on_change,
        );
    }));

    measure(Box::new(move |input| {
        let component_width = resolve_component_width(&args, input.parent_constraint);
        let (is_dragging, is_focused) = controller.with(|c| (c.is_dragging(), c.is_focused()));
        let base_handle_width = args.thumb_diameter.to_px();
        let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
        let handle_width = if is_dragging || is_focused {
            pressed_handle_width
        } else {
            base_handle_width
        };
        let resolved_layout = slider_layout_with_handle_width(&args, component_width, handle_width);
        let has_inset_icon = args.inset_icon.is_some();
        measure_slider(
            input,
            resolved_layout,
            clamped_value,
            has_inset_icon,
            handle_width,
            args.steps,
        )
    }));
}

fn measure_centered_slider(
    input: &MeasureInput,
    layout: CenteredSliderLayout,
    value: f32,
    handle_width: Px,
    steps: usize,
) -> Result<ComputedData, MeasurementError> {
    let self_width = layout.base.component_width;
    let self_height = layout.base.component_height;
    let track_y = layout.base.track_y;

    let left_inactive_id = input.children_ids[0];
    let active_id = input.children_ids[1];
    let right_inactive_id = input.children_ids[2];
    let mut current_index = 3;
    let tick_count = if steps == 0 { 0 } else { steps + 2 };
    let tick_ids = &input.children_ids[current_index..current_index + tick_count];
    current_index += tick_count;

    let (left_stop_id, right_stop_id) = if layout.base.show_stop_indicator {
        let left = input.children_ids[current_index];
        let right = input.children_ids[current_index + 1];
        current_index += 2;
        (Some(left), Some(right))
    } else {
        (None, None)
    };
    let handle_id = input.children_ids[current_index];

    let segments = layout.segments(value);

    // 1. Left Inactive
    input.measure_child(
        left_inactive_id,
        &Constraint::new(
            DimensionValue::Fixed(segments.left_inactive.1),
            DimensionValue::Fixed(layout.base.track_height),
        ),
    )?;
    input.place_child(
        left_inactive_id,
        PxPosition::new(segments.left_inactive.0, track_y),
    );

    // 2. Active
    input.measure_child(
        active_id,
        &Constraint::new(
            DimensionValue::Fixed(segments.active.1),
            DimensionValue::Fixed(layout.base.track_height),
        ),
    )?;
    input.place_child(active_id, PxPosition::new(segments.active.0, track_y));

    // 3. Right Inactive
    input.measure_child(
        right_inactive_id,
        &Constraint::new(
            DimensionValue::Fixed(segments.right_inactive.1),
            DimensionValue::Fixed(layout.base.track_height),
        ),
    )?;
    input.place_child(
        right_inactive_id,
        PxPosition::new(segments.right_inactive.0, track_y),
    );

    // 4. Handle
    let handle_offset = layout.base.center_child_offset(handle_width);
    input.measure_child(
        handle_id,
        &Constraint::new(
            DimensionValue::Fixed(handle_width),
            DimensionValue::Fixed(layout.base.handle_height),
        ),
    )?;
    input.place_child(
        handle_id,
        PxPosition::new(
            Px(segments.handle_center.x.0 - handle_offset.0),
            layout.base.handle_y,
        ),
    );

    if layout.base.show_stop_indicator {
        let (Some(left_stop_id), Some(right_stop_id)) = (left_stop_id, right_stop_id) else {
            return Err(MeasurementError::MeasureFnFailed(
                "Missing stop indicator children".to_string(),
            ));
        };
        // 5. Left Stop
        let stop_size = layout.base.stop_indicator_diameter;
        let stop_constraint = Constraint::new(
            DimensionValue::Fixed(stop_size),
            DimensionValue::Fixed(stop_size),
        );
        input.measure_child(left_stop_id, &stop_constraint)?;

        let stop_offset = layout.base.center_child_offset(stop_size);
        let stop_padding = layout.stop_indicator_offset();

        let left_stop_x = Px(stop_padding.0);

        input.place_child(
            left_stop_id,
            PxPosition::new(
                Px(left_stop_x.0 - stop_offset.0),
                layout.base.stop_indicator_y,
            ),
        );

        // 6. Right Stop
        input.measure_child(right_stop_id, &stop_constraint)?;
        let right_stop_x = Px(self_width.0 - stop_padding.0);

        input.place_child(
            right_stop_id,
            PxPosition::new(
                Px(right_stop_x.0 - stop_offset.0),
                layout.base.stop_indicator_y,
            ),
        );
    }

    if steps > 0 {
        let tick_size = layout.base.stop_indicator_diameter;
        let tick_constraint = Constraint::new(
            DimensionValue::Fixed(tick_size),
            DimensionValue::Fixed(tick_size),
        );
        let tick_offset = layout.base.center_child_offset(tick_size);
        let start_x = layout.base.handle_gap.to_f32() + handle_width.to_f32() / 2.0;
        for (i, tick_id) in tick_ids.iter().copied().enumerate() {
            input.measure_child(tick_id, &tick_constraint)?;
            let fraction = i as f32 / (steps as f32 + 1.0);
            let tick_center_x = start_x + fraction * layout.base.track_total_width.to_f32();
            input.place_child(
                tick_id,
                PxPosition::new(
                    Px(tick_center_x.round() as i32 - tick_offset.0),
                    layout.base.stop_indicator_y,
                ),
            );
        }
    }

    Ok(ComputedData {
        width: self_width,
        height: self_height,
    })
}

/// # centered_slider
///
/// Renders an interactive slider that originates from the center (0.5),
/// allowing selection of a value between 0.0 and 1.0. The active track extends
/// from the center to the handle, while inactive tracks fill the remaining
/// space.
///
/// ## Usage
///
/// Use for adjustments that have a neutral midpoint, such as balance controls
/// or deviation settings.
///
/// ## Parameters
///
/// - `args` — configures the slider's value, appearance, and callbacks; see
///   [`SliderArgs`].
/// - `controller` — optional controller; use
///   [`centered_slider_with_controller`] to supply one.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use std::sync::{Arc, Mutex};
/// use tessera_ui::{Dp, Modifier};
/// use tessera_ui_basic_components::modifier::ModifierExt as _;
/// use tessera_ui_basic_components::modifier::ModifierExt as _;
/// use tessera_ui_basic_components::slider::{SliderArgsBuilder, centered_slider};
/// let current_value = Arc::new(Mutex::new(0.5));
///
/// // Simulate a value change
/// {
///     let mut value_guard = current_value.lock().unwrap();
///     *value_guard = 0.75;
///     assert_eq!(*value_guard, 0.75);
/// }
///
/// centered_slider(
///     SliderArgsBuilder::default()
///         .modifier(Modifier::new().width(Dp(200.0)))
///         .value(*current_value.lock().unwrap())
///         .on_change(Arc::new(move |new_value| {
///             // In a real app, you would update your state here.
///             // For this example, we'll just check it after the simulated change.
///             println!("Centered slider value changed to: {}", new_value);
///         }))
///         .build()
///         .unwrap(),
/// );
///
/// // Simulate another value change and check the state
/// {
///     let mut value_guard = current_value.lock().unwrap();
///     *value_guard = 0.25;
///     assert_eq!(*value_guard, 0.25);
/// }
/// # }
/// # component();
/// ```
#[tessera]
pub fn centered_slider(args: impl Into<SliderArgs>) {
    let args: SliderArgs = args.into();
    let controller = remember(SliderController::new);
    centered_slider_with_controller(args, controller);
}

/// # centered_slider_with_controller
///
/// Controlled centered slider variant.
///
/// ## Usage
///
/// Use when you need to manage the centered slider's interactive state
/// externally.
///
/// ## Parameters
///
/// - `args` — configures the slider's value, appearance, and callbacks; see
///   [`SliderArgs`].
/// - `controller` — the slider controller to manage interactive state.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use std::sync::Arc;
/// use tessera_ui::{Dp, Modifier, remember};
/// use tessera_ui_basic_components::modifier::ModifierExt as _;
/// use tessera_ui_basic_components::slider::{
///     SliderArgsBuilder, SliderController, centered_slider_with_controller,
/// };
///
/// let controller = remember(SliderController::new);
/// centered_slider_with_controller(
///     SliderArgsBuilder::default()
///         .modifier(Modifier::new().width(Dp(200.0)))
///         .value(0.5)
///         .on_change(Arc::new(|new_value| {
///             println!("Centered slider value changed to: {}", new_value);
///         }))
///         .build()
///         .unwrap(),
///     controller,
/// );
/// # }
/// # component();
/// ```
#[tessera]
pub fn centered_slider_with_controller(
    args: impl Into<SliderArgs>,
    controller: State<SliderController>,
) {
    let args: SliderArgs = args.into();
    let initial_width = fallback_component_width(&args);
    let clamped_value = args.value.clamp(0.0, 1.0);
    let (is_dragging, is_focused) = controller.with(|c| (c.is_dragging(), c.is_focused()));
    let base_handle_width = args.thumb_diameter.to_px();
    let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
    let handle_width = if is_dragging || is_focused {
        pressed_handle_width
    } else {
        base_handle_width
    };
    let layout = CenteredSliderLayout {
        base: slider_layout_with_handle_width(&args, initial_width, handle_width),
    };
    let colors = slider_colors(&args);

    render_centered_tracks(layout, &colors);
    if args.steps > 0 {
        let active_start = clamped_value.min(0.5);
        let active_end = clamped_value.max(0.5);
        for fraction in tick_fractions(args.steps) {
            let is_active = fraction >= active_start && fraction <= active_end;
            let color = if is_active {
                colors.inactive_track
            } else {
                colors.active_track
            };
            render_tick(layout.base.stop_indicator_diameter, color);
        }
    }
    if layout.base.show_stop_indicator {
        render_centered_stops(layout, &colors);
    }
    render_handle(layout.base, handle_width, &colors);

    let cloned_args = args.clone();
    input_handler(Box::new(move |mut input| {
        let (is_dragging, is_focused) = controller.with(|c| (c.is_dragging(), c.is_focused()));
        let base_handle_width = cloned_args.thumb_diameter.to_px();
        let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
        let handle_width = if is_dragging || is_focused {
            pressed_handle_width
        } else {
            base_handle_width
        };
        let resolved_layout = CenteredSliderLayout {
            base: slider_layout_with_handle_width(
                &cloned_args,
                input.computed_data.width,
                handle_width,
            ),
        };
        handle_slider_state(&mut input, controller, &cloned_args, &resolved_layout.base);
        apply_slider_accessibility(
            &mut input,
            &cloned_args,
            clamped_value,
            &cloned_args.on_change,
        );
    }));

    measure(Box::new(move |input| {
        let component_width = resolve_component_width(&args, input.parent_constraint);
        let (is_dragging, is_focused) = controller.with(|c| (c.is_dragging(), c.is_focused()));
        let base_handle_width = args.thumb_diameter.to_px();
        let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
        let handle_width = if is_dragging || is_focused {
            pressed_handle_width
        } else {
            base_handle_width
        };
        let resolved_layout = CenteredSliderLayout {
            base: slider_layout_with_handle_width(&args, component_width, handle_width),
        };
        measure_centered_slider(
            input,
            resolved_layout,
            clamped_value,
            handle_width,
            args.steps,
        )
    }));
}

fn measure_range_slider(
    input: &MeasureInput,
    layout: RangeSliderLayout,
    args: RangeSliderMeasureArgs,
) -> Result<ComputedData, MeasurementError> {
    let self_width = layout.base.component_width;
    let self_height = layout.base.component_height;
    let track_y = layout.base.track_y;

    let left_inactive_id = input.children_ids[0];
    let active_id = input.children_ids[1];
    let right_inactive_id = input.children_ids[2];
    let mut current_index = 3;
    let tick_count = if args.steps == 0 { 0 } else { args.steps + 2 };
    let tick_ids = &input.children_ids[current_index..current_index + tick_count];
    current_index += tick_count;

    let (stop_start_id, stop_end_id) = if layout.base.show_stop_indicator {
        let start_id = input.children_ids.get(current_index).copied();
        let end_id = input.children_ids.get(current_index + 1).copied();
        current_index += 2;
        (start_id, end_id)
    } else {
        (None, None)
    };

    let handle_start_id = input.children_ids[current_index];
    let handle_end_id = input.children_ids[current_index + 1];

    let segments = layout.segments(
        args.start,
        args.end,
        args.start_handle_width,
        args.end_handle_width,
    );

    input.measure_child(
        left_inactive_id,
        &Constraint::new(
            DimensionValue::Fixed(segments.left_inactive.1),
            DimensionValue::Fixed(layout.base.track_height),
        ),
    )?;
    input.place_child(
        left_inactive_id,
        PxPosition::new(segments.left_inactive.0, track_y),
    );

    input.measure_child(
        active_id,
        &Constraint::new(
            DimensionValue::Fixed(segments.active.1),
            DimensionValue::Fixed(layout.base.track_height),
        ),
    )?;
    input.place_child(active_id, PxPosition::new(segments.active.0, track_y));

    input.measure_child(
        right_inactive_id,
        &Constraint::new(
            DimensionValue::Fixed(segments.right_inactive.1),
            DimensionValue::Fixed(layout.base.track_height),
        ),
    )?;
    input.place_child(
        right_inactive_id,
        PxPosition::new(segments.right_inactive.0, track_y),
    );

    let start_handle_constraint = Constraint::new(
        DimensionValue::Fixed(args.start_handle_width),
        DimensionValue::Fixed(layout.base.handle_height),
    );
    let end_handle_constraint = Constraint::new(
        DimensionValue::Fixed(args.end_handle_width),
        DimensionValue::Fixed(layout.base.handle_height),
    );
    let start_handle_offset = layout.base.center_child_offset(args.start_handle_width);
    let end_handle_offset = layout.base.center_child_offset(args.end_handle_width);

    input.measure_child(handle_start_id, &start_handle_constraint)?;
    input.place_child(
        handle_start_id,
        PxPosition::new(
            Px(segments.start_handle_center.x.0 - start_handle_offset.0),
            layout.base.handle_y,
        ),
    );

    input.measure_child(handle_end_id, &end_handle_constraint)?;
    input.place_child(
        handle_end_id,
        PxPosition::new(
            Px(segments.end_handle_center.x.0 - end_handle_offset.0),
            layout.base.handle_y,
        ),
    );

    let start_value = args.start;
    let end_value = args.end;
    set_range_thumb_accessibility(
        input,
        handle_start_id,
        RangeThumbAccessibilityArgs {
            key: "range_slider_start_thumb",
            label: args.accessibility_label.as_ref(),
            description: args.accessibility_description.as_ref(),
            fallback_description: "range start",
            steps: args.steps,
            disabled: args.disabled,
            value: start_value,
            min: 0.0,
            max: end_value,
            on_change: Arc::new({
                let on_change = args.on_change.clone();
                move |new_start| (on_change)((new_start, end_value))
            }),
        },
    );
    set_range_thumb_accessibility(
        input,
        handle_end_id,
        RangeThumbAccessibilityArgs {
            key: "range_slider_end_thumb",
            label: args.accessibility_label.as_ref(),
            description: args.accessibility_description.as_ref(),
            fallback_description: "range end",
            steps: args.steps,
            disabled: args.disabled,
            value: end_value,
            min: start_value,
            max: 1.0,
            on_change: Arc::new({
                let on_change = args.on_change.clone();
                move |new_end| (on_change)((start_value, new_end))
            }),
        },
    );

    if args.steps > 0 {
        let tick_size = layout.base.stop_indicator_diameter;
        let tick_constraint = Constraint::new(
            DimensionValue::Fixed(tick_size),
            DimensionValue::Fixed(tick_size),
        );
        let tick_offset = layout.base.center_child_offset(tick_size);

        let component_width = layout.base.component_width.to_f32();
        let gap = layout.base.handle_gap.to_f32();
        let start_half = args.start_handle_width.to_f32() / 2.0;
        let end_half = args.end_handle_width.to_f32() / 2.0;
        let track_total = (component_width - start_half - end_half - gap * 2.0).max(0.0);
        let start_x = gap + start_half;
        for (i, tick_id) in tick_ids.iter().copied().enumerate() {
            input.measure_child(tick_id, &tick_constraint)?;
            let fraction = i as f32 / (args.steps as f32 + 1.0);
            let tick_center_x = start_x + fraction * track_total;
            input.place_child(
                tick_id,
                PxPosition::new(
                    Px(tick_center_x.round() as i32 - tick_offset.0),
                    layout.base.stop_indicator_y,
                ),
            );
        }
    }

    if layout.base.show_stop_indicator {
        let (Some(stop_start_id), Some(stop_end_id)) = (stop_start_id, stop_end_id) else {
            return Err(MeasurementError::MeasureFnFailed(
                "Missing stop indicator children".to_string(),
            ));
        };

        let stop_size = layout.base.stop_indicator_diameter;
        let stop_constraint = Constraint::new(
            DimensionValue::Fixed(stop_size),
            DimensionValue::Fixed(stop_size),
        );
        input.measure_child(stop_start_id, &stop_constraint)?;

        let stop_offset = layout.base.center_child_offset(stop_size);
        let corner = layout.base.track_corner_radius.to_px();
        let start_stop_x = corner;

        input.place_child(
            stop_start_id,
            PxPosition::new(
                Px(start_stop_x.0 - stop_offset.0),
                layout.base.stop_indicator_y,
            ),
        );

        input.measure_child(stop_end_id, &stop_constraint)?;
        let end_stop_x = Px(self_width.0 - corner.0);

        input.place_child(
            stop_end_id,
            PxPosition::new(
                Px(end_stop_x.0 - stop_offset.0),
                layout.base.stop_indicator_y,
            ),
        );
    }

    Ok(ComputedData {
        width: self_width,
        height: self_height,
    })
}

/// # range_slider
///
/// Renders an interactive slider with two handles, allowing selection of a
/// range (start, end) between 0.0 and 1.0.
///
/// ## Usage
///
/// Use for filtering by range, setting minimum and maximum values, or defining
/// an interval.
///
/// ## Parameters
///
/// - `args` — configures the slider's range, appearance, and callbacks; see
///   [`RangeSliderArgs`].
/// - `controller` — optional controller; use [`range_slider_with_controller`]
///   to supply one.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use std::sync::{Arc, Mutex};
/// use tessera_ui::{Dp, Modifier};
/// use tessera_ui_basic_components::modifier::ModifierExt as _;
/// use tessera_ui_basic_components::slider::{RangeSliderArgsBuilder, range_slider};
/// let range_value = Arc::new(Mutex::new((0.2, 0.8)));
///
/// range_slider(
///     RangeSliderArgsBuilder::default()
///         .modifier(Modifier::new().width(Dp(200.0)))
///         .value(*range_value.lock().unwrap())
///         .on_change(Arc::new(move |(start, end)| {
///             println!("Range changed: {} - {}", start, end);
///         }))
///         .build()
///         .unwrap(),
/// );
/// # }
/// # component();
/// ```
#[tessera]
pub fn range_slider(args: impl Into<RangeSliderArgs>) {
    let args: RangeSliderArgs = args.into();
    let state = remember(RangeSliderController::new);
    range_slider_with_controller(args, state);
}

/// Controlled range slider variant.
#[tessera]
pub fn range_slider_with_controller(
    args: impl Into<RangeSliderArgs>,
    state: State<RangeSliderController>,
) {
    let args: RangeSliderArgs = args.into();
    let modifier = args.modifier;
    modifier.run(move || range_slider_with_controller_inner(args, state));
}

#[tessera]
fn range_slider_with_controller_inner(args: RangeSliderArgs, state: State<RangeSliderController>) {
    let dummy_slider_args = SliderArgsBuilder::default()
        .size(args.size)
        .show_stop_indicator(args.show_stop_indicator)
        .build()
        .expect("Failed to build dummy args");
    let initial_width = fallback_component_width(&dummy_slider_args);
    let dummy_for_measure = dummy_slider_args.clone();
    let layout = range_slider_layout(&args, initial_width);

    let start = args.value.0.clamp(0.0, 1.0);
    let end = args.value.1.clamp(start, 1.0);

    let base_handle_width = args.thumb_diameter.to_px();
    let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
    let (start_interacting, end_interacting) = state.with(|s| {
        (
            s.is_dragging_start || s.focus_start.is_focused(),
            s.is_dragging_end || s.focus_end.is_focused(),
        )
    });
    let start_handle_width = if start_interacting {
        pressed_handle_width
    } else {
        base_handle_width
    };
    let end_handle_width = if end_interacting {
        pressed_handle_width
    } else {
        base_handle_width
    };

    let colors = range_slider_colors(&args);

    render_range_tracks(layout, &colors);
    if args.steps > 0 {
        for fraction in tick_fractions(args.steps) {
            let is_active = fraction >= start && fraction <= end;
            let color = if is_active {
                colors.inactive_track
            } else {
                colors.active_track
            };
            render_tick(layout.base.stop_indicator_diameter, color);
        }
    }
    if layout.base.show_stop_indicator {
        render_range_stops(layout, &colors);
    }
    render_handle(layout.base, start_handle_width, &colors);
    render_handle(layout.base, end_handle_width, &colors);

    let cloned_args = args.clone();
    let start_val = start;
    let end_val = end;

    input_handler(Box::new(move |mut input| {
        let resolved_layout = range_slider_layout(&cloned_args, input.computed_data.width);
        let base_handle_width = cloned_args.thumb_diameter.to_px();
        let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
        let (start_interacting, end_interacting) = state.with(|s| {
            (
                s.is_dragging_start || s.focus_start.is_focused(),
                s.is_dragging_end || s.focus_end.is_focused(),
            )
        });
        let start_handle_width = if start_interacting {
            pressed_handle_width
        } else {
            base_handle_width
        };
        let end_handle_width = if end_interacting {
            pressed_handle_width
        } else {
            base_handle_width
        };
        handle_range_slider_state(
            &mut input,
            &state,
            &cloned_args,
            &resolved_layout.base,
            start_handle_width,
            end_handle_width,
        );
        apply_range_slider_accessibility(
            &mut input,
            &cloned_args,
            start_val,
            end_val,
            &cloned_args.on_change,
        );
    }));

    measure(Box::new(move |input| {
        let component_width = resolve_component_width(&dummy_for_measure, input.parent_constraint);
        let resolved_layout = range_slider_layout(&args, component_width);
        let base_handle_width = args.thumb_diameter.to_px();
        let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
        let (start_interacting, end_interacting) = state.with(|s| {
            (
                s.is_dragging_start || s.focus_start.is_focused(),
                s.is_dragging_end || s.focus_end.is_focused(),
            )
        });
        let start_handle_width = if start_interacting {
            pressed_handle_width
        } else {
            base_handle_width
        };
        let end_handle_width = if end_interacting {
            pressed_handle_width
        } else {
            base_handle_width
        };
        measure_range_slider(
            input,
            resolved_layout,
            RangeSliderMeasureArgs {
                start,
                end,
                start_handle_width,
                end_handle_width,
                steps: args.steps,
                disabled: args.disabled,
                accessibility_label: args.accessibility_label.clone(),
                accessibility_description: args.accessibility_description.clone(),
                on_change: args.on_change.clone(),
            },
        )
    }));
}
