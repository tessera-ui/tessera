//! An interactive slider component for selecting a value in a range.
//!
//! ## Usage
//!
//! Use to allow users to select a value from a continuous range.
use std::sync::Arc;

use derive_builder::Builder;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, MeasureInput, MeasurementError, Px,
    PxPosition, focus_state::Focus, remember, tessera,
};

use crate::{material_color, pipelines::image_vector::command::VectorTintMode};

use interaction::{
    apply_range_slider_accessibility, apply_slider_accessibility, handle_range_slider_state,
    handle_slider_state,
};
use layout::{
    CenteredSliderLayout, RangeSliderLayout, SliderLayout, centered_slider_layout,
    fallback_component_width, range_slider_layout, resolve_component_width, slider_layout,
};
use render::{
    render_active_segment, render_centered_stops, render_centered_tracks, render_focus,
    render_handle, render_inactive_segment, render_range_stops, render_range_tracks,
    render_stop_indicator,
};

pub use interaction::RangeSliderController;

mod interaction;
mod layout;
mod render;

const ACCESSIBILITY_STEP: f32 = 0.05;
const MIN_TOUCH_TARGET: Dp = Dp(40.0);
const HANDLE_GAP: Dp = Dp(6.0);
const STOP_INDICATOR_DIAMETER: Dp = Dp(4.0);

/// Stores the interactive state for the [`slider`] component, such as whether the slider is currently being dragged by the user.
pub(crate) struct SliderStateInner {
    /// True if the user is currently dragging the slider.
    pub is_dragging: bool,
    /// The focus handler for the slider.
    pub focus: Focus,
    /// True when the cursor is hovering inside the slider bounds.
    pub is_hovered: bool,
}

impl Default for SliderStateInner {
    fn default() -> Self {
        Self::new()
    }
}

impl SliderStateInner {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            focus: Focus::new(),
            is_hovered: false,
        }
    }
}

/// Controller for the `slider` component.
pub struct SliderController {
    inner: RwLock<SliderStateInner>,
}

impl SliderController {
    /// Creates a new slider controller.
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(SliderStateInner::new()),
        }
    }

    pub(crate) fn read(&self) -> RwLockReadGuard<'_, SliderStateInner> {
        self.inner.read()
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, SliderStateInner> {
        self.inner.write()
    }

    /// Returns whether the slider handle is currently being dragged.
    pub fn is_dragging(&self) -> bool {
        self.read().is_dragging
    }

    /// Manually sets the dragging flag. Useful for custom gesture integrations.
    pub fn set_dragging(&self, dragging: bool) {
        self.write().is_dragging = dragging;
    }

    /// Requests focus for the slider.
    pub fn request_focus(&self) {
        self.write().focus.request_focus();
    }

    /// Clears focus from the slider if it is currently focused.
    pub fn clear_focus(&self) {
        self.write().focus.unfocus();
    }

    /// Returns `true` if this slider currently holds focus.
    pub fn is_focused(&self) -> bool {
        self.read().focus.is_focused()
    }

    /// Returns `true` if the cursor is hovering over this slider.
    pub fn is_hovered(&self) -> bool {
        self.read().is_hovered
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
    /// The current value of the slider, ranging from 0.0 to 1.0.
    #[builder(default = "0.0")]
    pub value: f32,
    /// Callback function triggered when the slider's value changes.
    #[builder(default = "Arc::new(|_| {})")]
    pub on_change: Arc<dyn Fn(f32) + Send + Sync>,
    /// Size variant of the slider.
    #[builder(default)]
    pub size: SliderSize,
    /// Total width of the slider control.
    #[builder(default = "DimensionValue::Fixed(Dp(260.0).to_px())")]
    pub width: DimensionValue,
    /// The color of the active part of the track (progress fill).
    #[builder(default = "crate::material_color::global_material_scheme().primary")]
    pub active_track_color: Color,
    /// The color of the inactive part of the track (background).
    #[builder(default = "crate::material_color::global_material_scheme().secondary_container")]
    pub inactive_track_color: Color,
    /// The thickness of the handle indicator.
    #[builder(default = "Dp(4.0)")]
    pub thumb_diameter: Dp,
    /// Color of the handle indicator.
    #[builder(default = "crate::material_color::global_material_scheme().primary")]
    pub thumb_color: Color,
    /// Height of the handle focus layer (hover/drag halo).
    #[builder(default = "Dp(18.0)")]
    pub state_layer_diameter: Dp,
    /// Base color for the state layer; alpha will be adjusted per interaction state.
    #[builder(
        default = "crate::material_color::global_material_scheme().primary.with_alpha(0.18)"
    )]
    pub state_layer_color: Color,
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
    /// Optional icon content to display at the start of the slider (only for Medium sizes and above).
    #[builder(default, setter(strip_option, into))]
    pub inset_icon: Option<crate::icon::IconContent>,
}

/// Arguments for the `range_slider` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct RangeSliderArgs {
    /// The current range values (start, end), each between 0.0 and 1.0.
    #[builder(default = "(0.0, 1.0)")]
    pub value: (f32, f32),

    /// Callback function triggered when the range values change.
    #[builder(default = "Arc::new(|_| {})")]
    pub on_change: Arc<dyn Fn((f32, f32)) + Send + Sync>,

    /// Size variant of the slider.
    #[builder(default)]
    pub size: SliderSize,

    /// Total width of the slider control.
    #[builder(default = "DimensionValue::Fixed(Dp(260.0).to_px())")]
    pub width: DimensionValue,

    /// The color of the active part of the track (range fill).
    #[builder(default = "crate::material_color::global_material_scheme().primary")]
    pub active_track_color: Color,

    /// The color of the inactive part of the track (background).
    #[builder(default = "crate::material_color::global_material_scheme().secondary_container")]
    pub inactive_track_color: Color,

    /// The thickness of the handle indicators.
    #[builder(default = "Dp(4.0)")]
    pub thumb_diameter: Dp,

    /// Color of the handle indicators.
    #[builder(default = "crate::material_color::global_material_scheme().primary")]
    pub thumb_color: Color,

    /// Height of the handle focus layer.
    #[builder(default = "Dp(18.0)")]
    pub state_layer_diameter: Dp,

    /// Base color for the state layer.
    #[builder(
        default = "crate::material_color::global_material_scheme().primary.with_alpha(0.18)"
    )]
    pub state_layer_color: Color,

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
}

fn measure_slider(
    input: &MeasureInput,
    layout: SliderLayout,
    clamped_value: f32,
    has_inset_icon: bool,
) -> Result<ComputedData, MeasurementError> {
    let self_width = layout.component_width;
    let self_height = layout.component_height;

    let active_id = input.children_ids[0];
    let inactive_id = input.children_ids[1];

    // Order in render: active, inactive, [icon], focus, handle, [stop]
    let mut current_index = 2;

    let icon_id = if has_inset_icon {
        let id = input.children_ids.get(current_index).copied();
        current_index += 1;
        id
    } else {
        None
    };

    let focus_id = input.children_ids[current_index];
    current_index += 1;
    let handle_id = input.children_ids[current_index];
    current_index += 1;

    let stop_id = if layout.show_stop_indicator {
        input.children_ids.get(current_index).copied()
    } else {
        None
    };

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
            Px(active_width.0 + layout.handle_gap.0 * 2 + layout.handle_width.0),
            layout.track_y,
        ),
    );

    let focus_constraint = Constraint::new(
        DimensionValue::Fixed(layout.focus_width),
        DimensionValue::Fixed(layout.focus_height),
    );
    input.measure_child(focus_id, &focus_constraint)?;

    let handle_constraint = Constraint::new(
        DimensionValue::Fixed(layout.handle_width),
        DimensionValue::Fixed(layout.handle_height),
    );
    input.measure_child(handle_id, &handle_constraint)?;

    let handle_center = layout.handle_center(clamped_value);
    let focus_offset = layout.center_child_offset(layout.focus_width);
    input.place_child(
        focus_id,
        PxPosition::new(Px(handle_center.x.0 - focus_offset.0), layout.focus_y),
    );

    let handle_offset = layout.center_child_offset(layout.handle_width);
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
        let inactive_start = active_width.0 + layout.handle_gap.0 * 2 + layout.handle_width.0;
        let padding = Dp(8.0).to_px() - stop_size / Px(2);
        let stop_center_x = Px(inactive_start + inactive_width.0 - padding.0);
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

        // Icon placement: 8dp padding from left edge, vertically centered within the track
        let icon_padding = Dp(8.0).to_px();
        let icon_y = layout.track_y + Px((layout.track_height.0 - icon_measured.height.0) / 2);
        input.place_child(icon_id, PxPosition::new(icon_padding, icon_y));
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
    handle: Color,
    handle_focus: Color,
}

fn slider_colors(args: &SliderArgs, is_hovered: bool, is_dragging: bool) -> SliderColors {
    if args.disabled {
        let scheme = material_color::global_material_scheme();
        return SliderColors {
            active_track: scheme.on_surface.with_alpha(0.38),
            inactive_track: scheme.on_surface.with_alpha(0.12),
            handle: scheme.on_surface.with_alpha(0.38),
            handle_focus: Color::new(0.0, 0.0, 0.0, 0.0),
        };
    }

    let mut state_layer_alpha_scale = 0.0;
    if is_dragging {
        state_layer_alpha_scale = 1.0;
    } else if is_hovered {
        state_layer_alpha_scale = 0.7;
    }
    let base_state = args.state_layer_color;
    let state_layer_alpha = (base_state.a * state_layer_alpha_scale).clamp(0.0, 1.0);
    let handle_focus = Color::new(base_state.r, base_state.g, base_state.b, state_layer_alpha);

    SliderColors {
        active_track: args.active_track_color,
        inactive_track: args.inactive_track_color,
        handle: args.thumb_color,
        handle_focus,
    }
}

/// # slider
///
/// Renders an interactive slider with a bar-style handle for selecting a value between 0.0 and 1.0.
///
/// ## Usage
///
/// Use for settings like volume or brightness, or for any user-adjustable value.
///
/// ## Parameters
///
/// - `args` — configures the slider's value, appearance, and callbacks; see [`SliderArgs`].
/// - `controller` — optional; use [`slider_with_controller`] to provide your own controller.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{DimensionValue, Dp};
/// use tessera_ui_basic_components::slider::{slider, SliderArgsBuilder};
///
/// slider(
///     SliderArgsBuilder::default()
///         .width(DimensionValue::Fixed(Dp(200.0).to_px()))
///         .value(0.5)
///         .on_change(Arc::new(|new_value| {
///             // In a real app, you would update your state here.
///             println!("Slider value changed to: {}", new_value);
///         }))
///         .build()
///         .unwrap(),
/// );
/// ```
#[tessera]
pub fn slider(args: impl Into<SliderArgs>) {
    let args: SliderArgs = args.into();
    let controller = remember(SliderController::new);
    slider_with_controller(args, controller);
}

/// # slider_with_controller
///
/// Controlled slider variant
///
/// # Usage
///
/// Use when you need to manage the slider's interactive state externally.
///
/// # Parameters
///
/// - `args` — configures the slider's value, appearance, and callbacks; see [`SliderArgs`].
/// - `controller` — the slider controller to manage interactive state.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui::{DimensionValue, Dp, remember};
/// use tessera_ui_basic_components::slider::{slider_with_controller, SliderArgsBuilder, SliderController};
///
/// #[tessera]
/// fn foo() {
///     let controller = remember(|| SliderController::new());
///     slider_with_controller(
///        SliderArgsBuilder::default()
///            .width(DimensionValue::Fixed(Dp(200.0).to_px()))
///            .value(0.5)
///            .on_change(Arc::new(|new_value| {
///                println!("Slider value changed to: {}", new_value);
///            }))
///           .build()
///           .unwrap(),
///        controller.clone(),
///    );
/// }
#[tessera]
pub fn slider_with_controller(args: impl Into<SliderArgs>, controller: Arc<SliderController>) {
    let args: SliderArgs = args.into();
    let initial_width = fallback_component_width(&args);
    let layout = slider_layout(&args, initial_width);
    let clamped_value = args.value.clamp(0.0, 1.0);
    let state_snapshot = controller.read();
    let colors = slider_colors(&args, state_snapshot.is_hovered, state_snapshot.is_dragging);
    drop(state_snapshot);

    render_active_segment(layout, &colors);
    render_inactive_segment(layout, &colors);

    if let Some(icon_size) = layout.icon_size
        && let Some(inset_icon) = args.inset_icon.as_ref()
    {
        let scheme = material_color::global_material_scheme();
        let tint = if args.disabled {
            scheme.on_surface.with_alpha(0.38)
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

    render_focus(layout, &colors);
    render_handle(layout, &colors);
    if layout.show_stop_indicator {
        render_stop_indicator(layout, &colors);
    }

    let cloned_args = args.clone();
    let controller_clone = controller.clone();
    let clamped_value_for_accessibility = clamped_value;
    input_handler(Box::new(move |mut input| {
        let resolved_layout = slider_layout(&cloned_args, input.computed_data.width);
        handle_slider_state(
            &mut input,
            &controller_clone,
            &cloned_args,
            &resolved_layout,
        );
        apply_slider_accessibility(
            &mut input,
            &cloned_args,
            clamped_value_for_accessibility,
            &cloned_args.on_change,
        );
    }));

    measure(Box::new(move |input| {
        let component_width = resolve_component_width(&args, input.parent_constraint);
        let resolved_layout = slider_layout(&args, component_width);
        let has_inset_icon = args.inset_icon.is_some();
        measure_slider(input, resolved_layout, clamped_value, has_inset_icon)
    }));
}

fn measure_centered_slider(
    input: &MeasureInput,
    layout: CenteredSliderLayout,
    value: f32,
) -> Result<ComputedData, MeasurementError> {
    let self_width = layout.base.component_width;
    let self_height = layout.base.component_height;
    let track_y = layout.base.track_y;

    let left_inactive_id = input.children_ids[0];
    let active_id = input.children_ids[1];
    let right_inactive_id = input.children_ids[2];
    let focus_id = input.children_ids[3];
    let handle_id = input.children_ids[4];
    let left_stop_id = input.children_ids[5];
    let right_stop_id = input.children_ids[6];

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

    // 4. Focus
    let focus_offset = layout.base.center_child_offset(layout.base.focus_width);
    input.measure_child(
        focus_id,
        &Constraint::new(
            DimensionValue::Fixed(layout.base.focus_width),
            DimensionValue::Fixed(layout.base.focus_height),
        ),
    )?;
    input.place_child(
        focus_id,
        PxPosition::new(
            Px(segments.handle_center.x.0 - focus_offset.0),
            layout.base.focus_y,
        ),
    );

    // 5. Handle
    let handle_offset = layout.base.center_child_offset(layout.base.handle_width);
    input.measure_child(
        handle_id,
        &Constraint::new(
            DimensionValue::Fixed(layout.base.handle_width),
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
        // 6. Left Stop
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

        // 7. Right Stop
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

    Ok(ComputedData {
        width: self_width,
        height: self_height,
    })
}

/// # centered_slider
///
/// Renders an interactive slider that originates from the center (0.5), allowing selection of a value
/// between 0.0 and 1.0. The active track extends from the center to the handle, while inactive
/// tracks fill the remaining space.
///
/// ## Usage
///
/// Use for adjustments that have a neutral midpoint, such as balance controls or deviation settings.
///
/// ## Parameters
///
/// - `args` — configures the slider's value, appearance, and callbacks; see [`SliderArgs`].
/// - `controller` — optional controller; use [`centered_slider_with_controller`] to supply one.
///
/// ## Examples
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use tessera_ui::{DimensionValue, Dp};
/// use tessera_ui_basic_components::slider::{centered_slider, SliderArgsBuilder};
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
///         .width(DimensionValue::Fixed(Dp(200.0).to_px()))
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
/// ```
#[tessera]
pub fn centered_slider(args: impl Into<SliderArgs>) {
    let args: SliderArgs = args.into();
    let controller = remember(SliderController::new);
    centered_slider_with_controller(args, controller);
}

/// # centered_slider_with_controller
///
/// Controlled centered slider variant
///
/// # Usage
///
/// Use when you need to manage the slider's interactive state externally.
///
/// # Parameters
///
/// - `args` — configures the slider's value, appearance, and callbacks; see [`SliderArgs`].
/// - `controller` — the slider controller to manage interactive state.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui::{DimensionValue, Dp, remember};
/// use tessera_ui_basic_components::slider::{centered_slider_with_controller, SliderArgsBuilder, SliderController};
///
/// #[tessera]
/// fn foo() {
///     let controller = remember(|| SliderController::new());
///     centered_slider_with_controller(
///         SliderArgsBuilder::default()
///            .width(DimensionValue::Fixed(Dp(200.0).to_px()))
///            .value(0.5)
///            .on_change(Arc::new(|new_value| {
///                 println!("Centered slider value changed to: {}", new_value);
///         }))
///         .build()
///         .unwrap(),
///        controller.clone(),
///     );
/// }
#[tessera]
pub fn centered_slider_with_controller(
    args: impl Into<SliderArgs>,
    controller: Arc<SliderController>,
) {
    let args: SliderArgs = args.into();
    let initial_width = fallback_component_width(&args);
    let layout = centered_slider_layout(&args, initial_width);
    let clamped_value = args.value.clamp(0.0, 1.0);
    let state_snapshot = controller.read();
    let colors = slider_colors(&args, state_snapshot.is_hovered, state_snapshot.is_dragging);
    drop(state_snapshot);

    render_centered_tracks(layout, &colors);
    render_focus(layout.base, &colors);
    render_handle(layout.base, &colors);
    if layout.base.show_stop_indicator {
        render_centered_stops(layout, &colors);
    }

    let cloned_args = args.clone();
    let controller_clone = controller.clone();
    let clamped_value_for_accessibility = clamped_value;
    input_handler(Box::new(move |mut input| {
        let resolved_layout = centered_slider_layout(&cloned_args, input.computed_data.width);
        handle_slider_state(
            &mut input,
            &controller_clone,
            &cloned_args,
            &resolved_layout.base,
        );
        apply_slider_accessibility(
            &mut input,
            &cloned_args,
            clamped_value_for_accessibility,
            &cloned_args.on_change,
        );
    }));

    measure(Box::new(move |input| {
        let component_width = resolve_component_width(&args, input.parent_constraint);
        let resolved_layout = centered_slider_layout(&args, component_width);
        measure_centered_slider(input, resolved_layout, clamped_value)
    }));
}

fn measure_range_slider(
    input: &MeasureInput,
    layout: RangeSliderLayout,
    start: f32,
    end: f32,
) -> Result<ComputedData, MeasurementError> {
    let self_width = layout.base.component_width;
    let self_height = layout.base.component_height;
    let track_y = layout.base.track_y;

    let left_inactive_id = input.children_ids[0];
    let active_id = input.children_ids[1];
    let right_inactive_id = input.children_ids[2];
    let focus_start_id = input.children_ids[3];
    let focus_end_id = input.children_ids[4];
    let handle_start_id = input.children_ids[5];
    let handle_end_id = input.children_ids[6];
    let stop_start_id = input.children_ids[7];
    let stop_end_id = input.children_ids[8];

    let segments = layout.segments(start, end);

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

    let focus_constraint = Constraint::new(
        DimensionValue::Fixed(layout.base.focus_width),
        DimensionValue::Fixed(layout.base.focus_height),
    );
    let handle_constraint = Constraint::new(
        DimensionValue::Fixed(layout.base.handle_width),
        DimensionValue::Fixed(layout.base.handle_height),
    );
    let focus_offset = layout.base.center_child_offset(layout.base.focus_width);
    let handle_offset = layout.base.center_child_offset(layout.base.handle_width);

    input.measure_child(focus_start_id, &focus_constraint)?;
    input.place_child(
        focus_start_id,
        PxPosition::new(
            Px(segments.start_handle_center.x.0 - focus_offset.0),
            layout.base.focus_y,
        ),
    );

    input.measure_child(handle_start_id, &handle_constraint)?;
    input.place_child(
        handle_start_id,
        PxPosition::new(
            Px(segments.start_handle_center.x.0 - handle_offset.0),
            layout.base.handle_y,
        ),
    );

    input.measure_child(focus_end_id, &focus_constraint)?;
    input.place_child(
        focus_end_id,
        PxPosition::new(
            Px(segments.end_handle_center.x.0 - focus_offset.0),
            layout.base.focus_y,
        ),
    );

    input.measure_child(handle_end_id, &handle_constraint)?;
    input.place_child(
        handle_end_id,
        PxPosition::new(
            Px(segments.end_handle_center.x.0 - handle_offset.0),
            layout.base.handle_y,
        ),
    );

    if layout.base.show_stop_indicator {
        let stop_size = layout.base.stop_indicator_diameter;
        let stop_constraint = Constraint::new(
            DimensionValue::Fixed(stop_size),
            DimensionValue::Fixed(stop_size),
        );
        input.measure_child(stop_start_id, &stop_constraint)?;

        let stop_offset = layout.base.center_child_offset(stop_size);
        // We can reuse stop_indicator_offset logic if we expose it or reimplement it.
        // layout.base doesn't have it, CenteredSliderLayout does.
        // Let's reimplement simple padding: Dp(8.0) - size/2
        let padding = Dp(8.0).to_px() - stop_size / Px(2);
        let start_stop_x = Px(padding.0);

        input.place_child(
            stop_start_id,
            PxPosition::new(
                Px(start_stop_x.0 - stop_offset.0),
                layout.base.stop_indicator_y,
            ),
        );

        input.measure_child(stop_end_id, &stop_constraint)?;
        let end_stop_x = Px(self_width.0 - padding.0);

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
/// Renders an interactive slider with two handles, allowing selection of a range (start, end)
/// between 0.0 and 1.0.
///
/// ## Usage
///
/// Use for filtering by range, setting minimum and maximum values, or defining an interval.
///
/// ## Parameters
///
/// - `args` — configures the slider's range, appearance, and callbacks; see [`RangeSliderArgs`].
/// - `controller` — optional controller; use [`range_slider_with_controller`] to supply one.
///
/// ## Examples
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use tessera_ui::{DimensionValue, Dp};
/// use tessera_ui_basic_components::slider::{range_slider, RangeSliderArgsBuilder};
/// let range_value = Arc::new(Mutex::new((0.2, 0.8)));
///
/// range_slider(
///     RangeSliderArgsBuilder::default()
///         .width(DimensionValue::Fixed(Dp(200.0).to_px()))
///         .value(*range_value.lock().unwrap())
///         .on_change(Arc::new(move |(start, end)| {
///             println!("Range changed: {} - {}", start, end);
///         }))
///         .build()
///         .unwrap(),
/// );
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
    state: Arc<RangeSliderController>,
) {
    let args: RangeSliderArgs = args.into();
    // Convert RangeSliderArgs to SliderArgs for layout helpers where possible,
    // or rely on the dedicated range_slider_layout which handles this.
    let dummy_slider_args = SliderArgsBuilder::default()
        .width(args.width)
        .size(args.size)
        .build()
        .expect("Failed to build dummy args");
    let initial_width = fallback_component_width(&dummy_slider_args);
    let layout = range_slider_layout(&args, initial_width);

    let start = args.value.0.clamp(0.0, 1.0);
    let end = args.value.1.clamp(start, 1.0);

    let state_snapshot = state.read();
    // Determine colors based on interaction.
    // We check if *either* handle is interacted with to highlight the active tracks/handles?
    // Or ideally, we highlight specific handles.
    // For simplicity, let's use a unified color struct but apply focus colors selectively.

    let is_dragging_any = state_snapshot.is_dragging_start || state_snapshot.is_dragging_end;

    // Override colors from specific RangeSliderArgs
    // We need a helper to convert RangeSliderArgs colors to SliderColors if they differ
    // But for now we just reused the dummy args construction above which didn't copy colors.
    // Let's reconstruct colors properly.
    let mut state_layer_alpha_scale = 0.0;
    if is_dragging_any {
        state_layer_alpha_scale = 1.0;
    } else if state_snapshot.is_hovered {
        state_layer_alpha_scale = 0.7;
    }

    let base_state = args.state_layer_color;
    let state_layer_alpha = (base_state.a * state_layer_alpha_scale).clamp(0.0, 1.0);
    let handle_focus_color =
        Color::new(base_state.r, base_state.g, base_state.b, state_layer_alpha);

    let colors = if args.disabled {
        let scheme = material_color::global_material_scheme();
        SliderColors {
            active_track: scheme.on_surface.with_alpha(0.38),
            inactive_track: scheme.on_surface.with_alpha(0.12),
            handle: scheme.on_surface.with_alpha(0.38),
            handle_focus: Color::new(0.0, 0.0, 0.0, 0.0),
        }
    } else {
        SliderColors {
            active_track: args.active_track_color,
            inactive_track: args.inactive_track_color,
            handle: args.thumb_color,
            handle_focus: handle_focus_color,
        }
    };

    drop(state_snapshot);

    render_range_tracks(layout, &colors);

    // Render Start Focus & Handle
    render_focus(layout.base, &colors);
    // Note: render_focus uses layout.focus_width/height. Position is handled by measure/place.
    // But we need two focus indicators.

    // Render End Focus
    render_focus(layout.base, &colors);

    // Render Start Handle
    render_handle(layout.base, &colors);

    // Render End Handle
    render_handle(layout.base, &colors);

    if layout.base.show_stop_indicator {
        render_range_stops(layout, &colors);
    }

    let cloned_args = args.clone();
    let state_clone = state.clone();
    let start_val = start;
    let end_val = end;

    input_handler(Box::new(move |mut input| {
        let resolved_layout = range_slider_layout(&cloned_args, input.computed_data.width);
        handle_range_slider_state(
            &mut input,
            &state_clone,
            &cloned_args,
            &resolved_layout.base,
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
        let dummy_args_for_resolve = SliderArgsBuilder::default()
            .width(args.width)
            .size(args.size)
            .build()
            .expect("Failed to build dummy args");
        let component_width =
            resolve_component_width(&dummy_args_for_resolve, input.parent_constraint);
        let resolved_layout = range_slider_layout(&args, component_width);
        measure_range_slider(input, resolved_layout, start, end)
    }));
}
