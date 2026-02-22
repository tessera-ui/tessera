//! An interactive slider component for selecting a value in a range.
//!
//! ## Usage
//!
//! Use to allow users to select a value from a continuous range.
use derive_setters::Setters;
use tessera_ui::{
    CallbackWith, Color, ComputedData, Constraint, DimensionValue, Dp, InputHandlerInput,
    MeasurementError, Modifier, Px, PxPosition, State,
    accesskit::{Action, Role},
    focus_state::Focus,
    layout::{LayoutInput, LayoutOutput, LayoutSpec},
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

#[derive(Clone, PartialEq)]
struct RangeThumbAccessibilityArgs {
    key: &'static str,
    label: Option<String>,
    description: Option<String>,
    fallback_description: &'static str,
    steps: usize,
    disabled: bool,
    value: f32,
    min: f32,
    max: f32,
    on_change: CallbackWith<f32>,
}

#[derive(Clone, PartialEq)]
struct RangeSliderThumbArgs {
    thumb_layout: SliderLayout,
    handle_width: Px,
    colors: SliderColors,
    accessibility: RangeThumbAccessibilityArgs,
}

fn apply_range_thumb_accessibility(input: &InputHandlerInput, args: &RangeThumbAccessibilityArgs) {
    let mut builder = input.accessibility().role(Role::Slider).key(args.key);

    if let Some(label) = args.label.as_ref() {
        builder = builder.label(label.clone());
    }

    let description = args
        .description
        .as_ref()
        .map(|d| format!("{d} ({})", args.fallback_description))
        .unwrap_or_else(|| args.fallback_description.to_string());
    builder = builder.description(description);

    builder = builder
        .numeric_value(args.value as f64)
        .numeric_range(args.min as f64, args.max as f64);

    if args.disabled {
        builder = builder.disabled();
    } else {
        builder = builder
            .focusable()
            .action(Action::Increment)
            .action(Action::Decrement);
    }

    builder.commit();

    if args.disabled {
        return;
    }

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
    input.set_accessibility_action_handler(move |action| {
        let next = match action {
            Action::Increment => value + delta,
            Action::Decrement => value - delta,
            _ => return,
        };
        let next = snap_fraction(next, steps).clamp(min, max);
        on_change.call(next);
    });
}

#[tessera]
fn range_slider_thumb_node(args: &RangeSliderThumbArgs) {
    render_handle(args.thumb_layout, args.handle_width, &args.colors);
    let accessibility = args.accessibility.clone();

    input_handler(move |input| {
        apply_range_thumb_accessibility(&input, &accessibility);
    });
}

struct RangeSliderMeasureArgs {
    start: f32,
    end: f32,
    start_handle_width: Px,
    end_handle_width: Px,
    steps: usize,
}

#[derive(Clone)]
struct SliderLayoutSpec {
    args: SliderArgs,
    clamped_value: f32,
    handle_width: Px,
    has_inset_icon: bool,
}

impl PartialEq for SliderLayoutSpec {
    fn eq(&self, other: &Self) -> bool {
        self.args.size == other.args.size
            && self.args.show_stop_indicator == other.args.show_stop_indicator
            && self.args.steps == other.args.steps
            && self.clamped_value == other.clamped_value
            && self.handle_width == other.handle_width
            && self.has_inset_icon == other.has_inset_icon
    }
}

impl LayoutSpec for SliderLayoutSpec {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let component_width = resolve_component_width(&self.args, input.parent_constraint());
        let resolved_layout =
            slider_layout_with_handle_width(&self.args, component_width, self.handle_width);
        measure_slider(
            input,
            output,
            resolved_layout,
            self.clamped_value,
            self.has_inset_icon,
            self.handle_width,
            self.args.steps,
        )
    }
}

#[derive(Clone)]
struct CenteredSliderLayoutSpec {
    args: SliderArgs,
    clamped_value: f32,
    handle_width: Px,
}

impl PartialEq for CenteredSliderLayoutSpec {
    fn eq(&self, other: &Self) -> bool {
        self.args.size == other.args.size
            && self.args.show_stop_indicator == other.args.show_stop_indicator
            && self.args.steps == other.args.steps
            && self.clamped_value == other.clamped_value
            && self.handle_width == other.handle_width
    }
}

impl LayoutSpec for CenteredSliderLayoutSpec {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let component_width = resolve_component_width(&self.args, input.parent_constraint());
        let resolved_layout = CenteredSliderLayout {
            base: slider_layout_with_handle_width(&self.args, component_width, self.handle_width),
        };
        measure_centered_slider(
            input,
            output,
            resolved_layout,
            self.clamped_value,
            self.handle_width,
            self.args.steps,
        )
    }
}

#[derive(Clone)]
struct RangeSliderLayoutSpec {
    args: RangeSliderArgs,
    slider: SliderArgs,
    start: f32,
    end: f32,
    start_handle_width: Px,
    end_handle_width: Px,
}

impl PartialEq for RangeSliderLayoutSpec {
    fn eq(&self, other: &Self) -> bool {
        self.args.size == other.args.size
            && self.args.show_stop_indicator == other.args.show_stop_indicator
            && self.args.steps == other.args.steps
            && self.start == other.start
            && self.end == other.end
            && self.start_handle_width == other.start_handle_width
            && self.end_handle_width == other.end_handle_width
    }
}

impl LayoutSpec for RangeSliderLayoutSpec {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let component_width = resolve_component_width(&self.slider, input.parent_constraint());
        let resolved_layout = range_slider_layout(&self.args, component_width);
        measure_range_slider(
            input,
            output,
            resolved_layout,
            RangeSliderMeasureArgs {
                start: self.start,
                end: self.end,
                start_handle_width: self.start_handle_width,
                end_handle_width: self.end_handle_width,
                steps: self.args.steps,
            },
        )
    }
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
#[derive(PartialEq, Clone, Setters)]
pub struct SliderArgs {
    /// Modifier chain applied to the slider subtree.
    pub modifier: Modifier,
    /// The current value of the slider, ranging from 0.0 to 1.0.
    pub value: f32,
    /// Callback function triggered when the slider's value changes.
    #[setters(skip)]
    pub on_change: CallbackWith<f32>,
    /// Size variant of the slider.
    pub size: SliderSize,
    /// The color of the active part of the track (progress fill).
    pub active_track_color: Color,
    /// The color of the inactive part of the track (background).
    pub inactive_track_color: Color,
    /// The thickness of the handle indicator.
    pub thumb_diameter: Dp,
    /// Color of the handle indicator.
    pub thumb_color: Color,
    /// Disable interaction.
    pub disabled: bool,
    /// Optional accessibility label read by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
    /// Whether to show the stop indicators at the ends of the track.
    pub show_stop_indicator: bool,
    /// Number of discrete steps between 0.0 and 1.0.
    ///
    /// When set to a value greater than 0, the slider value snaps to
    /// `steps + 2` evenly spaced tick positions (including both ends).
    pub steps: usize,
    /// Optional icon content to display at the start of the slider (only for
    /// Medium sizes and above).
    #[setters(strip_option, into)]
    pub inset_icon: Option<crate::icon::IconContent>,
    /// Optional external controller for drag and focus state.
    ///
    /// When this is `None`, `slider` and `centered_slider` create and own an
    /// internal controller.
    #[setters(skip)]
    pub controller: Option<State<SliderController>>,
}

impl SliderArgs {
    /// Sets the on_change handler.
    pub fn on_change<F>(mut self, on_change: F) -> Self
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.on_change = CallbackWith::new(on_change);
        self
    }

    /// Sets the on_change handler using a shared callback.
    pub fn on_change_shared(mut self, on_change: impl Into<CallbackWith<f32>>) -> Self {
        self.on_change = on_change.into();
        self
    }

    /// Sets an external slider controller.
    pub fn controller(mut self, controller: State<SliderController>) -> Self {
        self.controller = Some(controller);
        self
    }
}

impl Default for SliderArgs {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        Self {
            modifier: Modifier::new(),
            value: 0.0,
            on_change: CallbackWith::new(|_| {}),
            size: SliderSize::default(),
            active_track_color: scheme.primary,
            inactive_track_color: scheme.secondary_container,
            thumb_diameter: Dp(4.0),
            thumb_color: scheme.primary,
            disabled: false,
            accessibility_label: None,
            accessibility_description: None,
            show_stop_indicator: true,
            steps: 0,
            inset_icon: None,
            controller: None,
        }
    }
}
/// Arguments for the `range_slider` component.
#[derive(PartialEq, Clone, Setters)]
pub struct RangeSliderArgs {
    /// Modifier chain applied to the range slider subtree.
    pub modifier: Modifier,
    /// The current range values (start, end), each between 0.0 and 1.0.
    pub value: (f32, f32),

    /// Callback function triggered when the range values change.
    #[setters(skip)]
    pub on_change: CallbackWith<(f32, f32)>,

    /// Size variant of the slider.
    pub size: SliderSize,

    /// The color of the active part of the track (range fill).
    pub active_track_color: Color,

    /// The color of the inactive part of the track (background).
    pub inactive_track_color: Color,

    /// The thickness of the handle indicators.
    pub thumb_diameter: Dp,

    /// Color of the handle indicators.
    pub thumb_color: Color,

    /// Disable interaction.
    pub disabled: bool,
    /// Optional accessibility label.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,

    /// Whether to show the stop indicators at the ends of the track.
    pub show_stop_indicator: bool,
    /// Number of discrete steps between 0.0 and 1.0.
    ///
    /// When set to a value greater than 0, the slider values snap to
    /// `steps + 2` evenly spaced tick positions (including both ends).
    pub steps: usize,
    /// Optional external range slider controller.
    ///
    /// When this is `None`, `range_slider` creates and owns an internal
    /// controller.
    #[setters(skip)]
    pub controller: Option<State<RangeSliderController>>,
}

impl RangeSliderArgs {
    /// Sets the on_change handler.
    pub fn on_change<F>(mut self, on_change: F) -> Self
    where
        F: Fn((f32, f32)) + Send + Sync + 'static,
    {
        self.on_change = CallbackWith::new(on_change);
        self
    }

    /// Sets the on_change handler using a shared callback.
    pub fn on_change_shared(mut self, on_change: impl Into<CallbackWith<(f32, f32)>>) -> Self {
        self.on_change = on_change.into();
        self
    }

    /// Sets an external range slider controller.
    pub fn controller(mut self, controller: State<RangeSliderController>) -> Self {
        self.controller = Some(controller);
        self
    }
}

impl Default for RangeSliderArgs {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        Self {
            modifier: Modifier::new(),
            value: (0.0, 1.0),
            on_change: CallbackWith::new(|_| {}),
            size: SliderSize::default(),
            active_track_color: scheme.primary,
            inactive_track_color: scheme.secondary_container,
            thumb_diameter: Dp(4.0),
            thumb_color: scheme.primary,
            disabled: false,
            accessibility_label: None,
            accessibility_description: None,
            show_stop_indicator: true,
            steps: 0,
            controller: None,
        }
    }
}

fn measure_slider(
    input: &LayoutInput<'_>,
    output: &mut LayoutOutput<'_>,
    layout: SliderLayout,
    clamped_value: f32,
    has_inset_icon: bool,
    handle_width: Px,
    steps: usize,
) -> Result<ComputedData, MeasurementError> {
    let self_width = layout.component_width;
    let self_height = layout.component_height;

    let active_id = input.children_ids()[0];
    let inactive_id = input.children_ids()[1];

    // Order in render: active, inactive, [icon], [ticks], [stop], handle
    let mut current_index = 2;

    let icon_id = if has_inset_icon {
        let id = input.children_ids().get(current_index).copied();
        current_index += 1;
        id
    } else {
        None
    };

    let tick_count = if steps == 0 { 0 } else { steps + 2 };
    let tick_ids = &input.children_ids()[current_index..current_index + tick_count];
    current_index += tick_count;

    let stop_id = if layout.show_stop_indicator {
        let id = input.children_ids().get(current_index).copied();
        current_index += 1;
        id
    } else {
        None
    };

    let handle_id = input.children_ids()[current_index];

    let active_width = layout.active_width(clamped_value);
    let inactive_width = layout.inactive_width(clamped_value);

    let active_constraint = Constraint::new(
        DimensionValue::Fixed(active_width),
        DimensionValue::Fixed(layout.track_height),
    );
    input.measure_child(active_id, &active_constraint)?;
    output.place_child(active_id, PxPosition::new(Px(0), layout.track_y));

    let inactive_constraint = Constraint::new(
        DimensionValue::Fixed(inactive_width),
        DimensionValue::Fixed(layout.track_height),
    );
    input.measure_child(inactive_id, &inactive_constraint)?;
    output.place_child(
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
    output.place_child(
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
        output.place_child(
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
        output.place_child(icon_id, PxPosition::new(icon_padding, icon_y));
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
            output.place_child(
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

#[derive(Clone, PartialEq, Copy)]
struct SliderColors {
    active_track: Color,
    inactive_track: Color,
    thumb: Color,
}

fn slider_colors(args: &SliderArgs) -> SliderColors {
    if args.disabled {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
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
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
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
/// - `controller` — optional; use [`slider`] to provide your own controller.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::modifier::ModifierExt as _;
/// use tessera_components::slider::{SliderArgs, slider};
/// use tessera_ui::{Dp, Modifier};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # let args = tessera_components::theme::MaterialThemeProviderArgs::new(
/// #     || MaterialTheme::default(),
/// #     || {
/// slider(
///     &SliderArgs::default()
///         .modifier(Modifier::new().width(Dp(200.0)))
///         .value(0.5)
///         .on_change(|new_value| {
///             // In a real app, you would update your state here.
///             println!("Slider value changed to: {}", new_value);
///         }),
/// );
/// #     },
/// # );
/// # material_theme(&args);
/// # }
/// # component();
/// ```
#[tessera]
pub fn slider(args: &SliderArgs) {
    let args: SliderArgs = args.clone();
    let controller = args
        .controller
        .unwrap_or_else(|| remember(SliderController::new));
    slider_node(args, controller);
}

fn slider_node(slider_args: SliderArgs, controller: State<SliderController>) {
    let modifier = slider_args.modifier.clone();
    modifier.run(move || {
        let mut inner_args = slider_args.clone();
        inner_args.controller = Some(controller);
        slider_inner_node(&inner_args);
    });
}

#[tessera]
fn slider_inner_node(args: &SliderArgs) {
    let args: SliderArgs = args.clone();
    let controller = args
        .controller
        .expect("slider_inner_node requires controller to be set");
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
    let slider_layout = slider_layout_with_handle_width(&args, initial_width, handle_width);
    let colors = slider_colors(&args);

    render_active_segment(slider_layout, &colors);
    render_inactive_segment(slider_layout, &colors);

    if let Some(icon_size) = slider_layout.icon_size
        && let Some(inset_icon) = args.inset_icon.as_ref()
    {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        let tint = if args.disabled {
            scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTENT)
        } else {
            scheme.on_primary
        };

        crate::icon::icon(
            &crate::icon::IconArgs::from(inset_icon.clone())
                .tint(tint)
                .tint_mode(VectorTintMode::Solid)
                .size(icon_size),
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
            render_tick(slider_layout.stop_indicator_diameter, color);
        }
    }
    if slider_layout.show_stop_indicator {
        render_stop_indicator(slider_layout, &colors);
    }
    render_handle(slider_layout, handle_width, &colors);

    let cloned_args = args.clone();
    input_handler(move |mut input| {
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
    });

    let has_inset_icon = args.inset_icon.is_some();
    layout(SliderLayoutSpec {
        args,
        clamped_value,
        handle_width,
        has_inset_icon,
    });
}

fn measure_centered_slider(
    input: &LayoutInput<'_>,
    output: &mut LayoutOutput<'_>,
    layout: CenteredSliderLayout,
    value: f32,
    handle_width: Px,
    steps: usize,
) -> Result<ComputedData, MeasurementError> {
    let self_width = layout.base.component_width;
    let self_height = layout.base.component_height;
    let track_y = layout.base.track_y;

    let left_inactive_id = input.children_ids()[0];
    let active_id = input.children_ids()[1];
    let right_inactive_id = input.children_ids()[2];
    let mut current_index = 3;
    let tick_count = if steps == 0 { 0 } else { steps + 2 };
    let tick_ids = &input.children_ids()[current_index..current_index + tick_count];
    current_index += tick_count;

    let (left_stop_id, right_stop_id) = if layout.base.show_stop_indicator {
        let left = input.children_ids()[current_index];
        let right = input.children_ids()[current_index + 1];
        current_index += 2;
        (Some(left), Some(right))
    } else {
        (None, None)
    };
    let handle_id = input.children_ids()[current_index];

    let segments = layout.segments(value);

    // 1. Left Inactive
    input.measure_child(
        left_inactive_id,
        &Constraint::new(
            DimensionValue::Fixed(segments.left_inactive.1),
            DimensionValue::Fixed(layout.base.track_height),
        ),
    )?;
    output.place_child(
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
    output.place_child(active_id, PxPosition::new(segments.active.0, track_y));

    // 3. Right Inactive
    input.measure_child(
        right_inactive_id,
        &Constraint::new(
            DimensionValue::Fixed(segments.right_inactive.1),
            DimensionValue::Fixed(layout.base.track_height),
        ),
    )?;
    output.place_child(
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
    output.place_child(
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

        output.place_child(
            left_stop_id,
            PxPosition::new(
                Px(left_stop_x.0 - stop_offset.0),
                layout.base.stop_indicator_y,
            ),
        );

        // 6. Right Stop
        input.measure_child(right_stop_id, &stop_constraint)?;
        let right_stop_x = Px(self_width.0 - stop_padding.0);

        output.place_child(
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
            output.place_child(
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
/// - `controller` — optional controller; use [`centered_slider`] to supply one.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use std::sync::{Arc, Mutex};
/// use tessera_components::modifier::ModifierExt as _;
/// use tessera_components::slider::{SliderArgs, centered_slider};
/// use tessera_ui::{Dp, Modifier};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// let current_value = Arc::new(Mutex::new(0.5));
///
/// // Simulate a value change
/// {
///     let mut value_guard = current_value.lock().unwrap();
///     *value_guard = 0.75;
///     assert_eq!(*value_guard, 0.75);
/// }
/// let value_for_slider = current_value.clone();
///
/// # let args = tessera_components::theme::MaterialThemeProviderArgs::new(
/// #     || MaterialTheme::default(),
/// #     move || {
/// centered_slider(
///     &SliderArgs::default()
///         .modifier(Modifier::new().width(Dp(200.0)))
///         .value(*value_for_slider.lock().unwrap())
///         .on_change(move |new_value| {
///             // In a real app, you would update your state here.
///             // For this example, we'll just check it after the simulated change.
///             println!("Centered slider value changed to: {}", new_value);
///         }),
/// );
/// #     },
/// # );
/// # material_theme(&args);
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
pub fn centered_slider(args: &SliderArgs) {
    let mut args: SliderArgs = args.clone();
    let controller = args
        .controller
        .unwrap_or_else(|| remember(SliderController::new));
    args.controller = Some(controller);
    centered_slider_node(&args);
}

#[tessera]
fn centered_slider_node(args: &SliderArgs) {
    let args = args.clone();
    let controller = args
        .controller
        .expect("centered_slider_node requires controller to be set");
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
    let centered_layout = CenteredSliderLayout {
        base: slider_layout_with_handle_width(&args, initial_width, handle_width),
    };
    let colors = slider_colors(&args);

    render_centered_tracks(centered_layout, &colors);
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
            render_tick(centered_layout.base.stop_indicator_diameter, color);
        }
    }
    if centered_layout.base.show_stop_indicator {
        render_centered_stops(centered_layout, &colors);
    }
    render_handle(centered_layout.base, handle_width, &colors);

    let cloned_args = args.clone();
    input_handler(move |mut input| {
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
    });

    layout(CenteredSliderLayoutSpec {
        args,
        clamped_value,
        handle_width,
    });
}

fn measure_range_slider(
    input: &LayoutInput<'_>,
    output: &mut LayoutOutput<'_>,
    layout: RangeSliderLayout,
    args: RangeSliderMeasureArgs,
) -> Result<ComputedData, MeasurementError> {
    let self_width = layout.base.component_width;
    let self_height = layout.base.component_height;
    let track_y = layout.base.track_y;

    let left_inactive_id = input.children_ids()[0];
    let active_id = input.children_ids()[1];
    let right_inactive_id = input.children_ids()[2];
    let mut current_index = 3;
    let tick_count = if args.steps == 0 { 0 } else { args.steps + 2 };
    let tick_ids = &input.children_ids()[current_index..current_index + tick_count];
    current_index += tick_count;

    let (stop_start_id, stop_end_id) = if layout.base.show_stop_indicator {
        let start_id = input.children_ids().get(current_index).copied();
        let end_id = input.children_ids().get(current_index + 1).copied();
        current_index += 2;
        (start_id, end_id)
    } else {
        (None, None)
    };

    let handle_start_id = input.children_ids()[current_index];
    let handle_end_id = input.children_ids()[current_index + 1];

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
    output.place_child(
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
    output.place_child(active_id, PxPosition::new(segments.active.0, track_y));

    input.measure_child(
        right_inactive_id,
        &Constraint::new(
            DimensionValue::Fixed(segments.right_inactive.1),
            DimensionValue::Fixed(layout.base.track_height),
        ),
    )?;
    output.place_child(
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
    output.place_child(
        handle_start_id,
        PxPosition::new(
            Px(segments.start_handle_center.x.0 - start_handle_offset.0),
            layout.base.handle_y,
        ),
    );

    input.measure_child(handle_end_id, &end_handle_constraint)?;
    output.place_child(
        handle_end_id,
        PxPosition::new(
            Px(segments.end_handle_center.x.0 - end_handle_offset.0),
            layout.base.handle_y,
        ),
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
            output.place_child(
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

        output.place_child(
            stop_start_id,
            PxPosition::new(
                Px(start_stop_x.0 - stop_offset.0),
                layout.base.stop_indicator_y,
            ),
        );

        input.measure_child(stop_end_id, &stop_constraint)?;
        let end_stop_x = Px(self_width.0 - corner.0);

        output.place_child(
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
/// - `controller` — optional controller; use [`range_slider`] to supply one.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use std::sync::{Arc, Mutex};
/// use tessera_components::modifier::ModifierExt as _;
/// use tessera_components::slider::{RangeSliderArgs, range_slider};
/// use tessera_ui::{Dp, Modifier};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// let range_value = Arc::new(Mutex::new((0.2, 0.8)));
///
/// # let args = tessera_components::theme::MaterialThemeProviderArgs::new(
/// #     || MaterialTheme::default(),
/// #     move || {
/// range_slider(
///     &RangeSliderArgs::default()
///         .modifier(Modifier::new().width(Dp(200.0)))
///         .value(*range_value.lock().unwrap())
///         .on_change(move |(start, end)| {
///             println!("Range changed: {} - {}", start, end);
///         }),
/// );
/// #     },
/// # );
/// # material_theme(&args);
/// # }
/// # component();
/// ```
#[tessera]
pub fn range_slider(args: &RangeSliderArgs) {
    let args: RangeSliderArgs = args.clone();
    let state = args
        .controller
        .unwrap_or_else(|| remember(RangeSliderController::new));
    range_slider_node(args, state);
}

fn range_slider_node(args: RangeSliderArgs, state: State<RangeSliderController>) {
    let modifier = args.modifier.clone();
    modifier.run(move || {
        let mut inner_args = args.clone();
        inner_args.controller = Some(state);
        range_slider_inner_node(&inner_args);
    });
}

#[tessera]
fn range_slider_inner_node(args: &RangeSliderArgs) {
    let args: RangeSliderArgs = args.clone();
    let state = args
        .controller
        .expect("range_slider_inner_node requires controller to be set");
    let dummy_slider_args = SliderArgs::default()
        .size(args.size)
        .show_stop_indicator(args.show_stop_indicator);
    let initial_width = fallback_component_width(&dummy_slider_args);
    let dummy_for_measure = dummy_slider_args.clone();
    let range_layout = range_slider_layout(&args, initial_width);

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

    render_range_tracks(range_layout, &colors);
    if args.steps > 0 {
        for fraction in tick_fractions(args.steps) {
            let is_active = fraction >= start && fraction <= end;
            let color = if is_active {
                colors.inactive_track
            } else {
                colors.active_track
            };
            render_tick(range_layout.base.stop_indicator_diameter, color);
        }
    }
    if range_layout.base.show_stop_indicator {
        render_range_stops(range_layout, &colors);
    }
    let start_thumb_args = RangeSliderThumbArgs {
        thumb_layout: range_layout.base,
        handle_width: start_handle_width,
        colors,
        accessibility: RangeThumbAccessibilityArgs {
            key: "range_slider_start_thumb",
            label: args.accessibility_label.clone(),
            description: args.accessibility_description.clone(),
            fallback_description: "range start",
            steps: args.steps,
            disabled: args.disabled,
            value: start,
            min: 0.0,
            max: end,
            on_change: CallbackWith::new({
                let on_change = args.on_change.clone();
                move |new_start| on_change.call((new_start, end))
            }),
        },
    };
    range_slider_thumb_node(&start_thumb_args);

    let end_thumb_args = RangeSliderThumbArgs {
        thumb_layout: range_layout.base,
        handle_width: end_handle_width,
        colors,
        accessibility: RangeThumbAccessibilityArgs {
            key: "range_slider_end_thumb",
            label: args.accessibility_label.clone(),
            description: args.accessibility_description.clone(),
            fallback_description: "range end",
            steps: args.steps,
            disabled: args.disabled,
            value: end,
            min: start,
            max: 1.0,
            on_change: CallbackWith::new({
                let on_change = args.on_change.clone();
                move |new_end| on_change.call((start, new_end))
            }),
        },
    };
    range_slider_thumb_node(&end_thumb_args);

    let cloned_args = args.clone();
    let start_val = start;
    let end_val = end;

    input_handler(move |mut input| {
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
    });

    layout(RangeSliderLayoutSpec {
        args,
        slider: dummy_for_measure,
        start,
        end,
        start_handle_width,
        end_handle_width,
    });
}
