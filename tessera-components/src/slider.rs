//! An interactive slider component for selecting a value in a range.
//!
//! ## Usage
//!
//! Use to allow users to select a value from a continuous range.
use tessera_foundation::gesture::{DragRecognizer, TapRecognizer};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, CallbackWith, Color,
    ComputedData, Constraint, Dp, FocusProperties, FocusRequester, LayoutResult, MeasurementError,
    Modifier, PointerInput, PointerInputModifierNode, Px, PxPosition, SemanticsModifierNode, State,
    accesskit::{Action, Role},
    layout::{MeasureScope, layout},
    modifier::{CursorModifierExt as _, FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
};

use crate::{
    icon::icon,
    image_vector::TintMode,
    modifier::{InteractionState, InteractionVisualState},
    painter::Painter,
    theme::{MaterialAlpha, MaterialColorScheme, MaterialTheme},
};

use axis::{SliderAxis, SliderMeasureScope};
use interaction::{
    RangeSliderHandleWidths, SliderKeyboardModel, SliderKeyboardModifierNode,
    apply_range_slider_semantics, apply_slider_semantics, handle_range_slider_state,
    handle_slider_state, snap_fraction,
};
use layout::{
    CenteredSliderLayout, RangeSliderLayout, SliderLayout, fallback_component_width,
    range_slider_layout, resolve_component_width, slider_layout_with_handle_width,
};
use render::{
    active_segment, centered_stops, centered_tracks, inactive_segment, range_stops, range_tracks,
    slider_handle, slider_tick, stop_indicator,
};

pub use indicator::{SliderValueIndicatorVisibility, slider_value_indicator_provider};
pub use interaction::RangeSliderController;

mod axis;
mod indicator;
mod interaction;
mod layout;
#[cfg(test)]
mod orientation_tests;
mod render;

const ACCESSIBILITY_STEP: f32 = 0.05;
const MIN_TOUCH_TARGET: Dp = Dp(40.0);
const HANDLE_GAP: Dp = Dp(6.0);
const STOP_INDICATOR_DIAMETER: Dp = Dp(4.0);

/// The physical axis used by a slider.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SliderOrientation {
    /// Values increase from left to right unless the direction is reversed.
    #[default]
    Horizontal,
    /// Values increase from top to bottom unless the direction is reversed.
    Vertical,
}

fn default_value_label(value: f32) -> String {
    format!("{:.0}", (value * 100.0).round())
}

fn tick_fractions(steps: usize) -> Vec<f32> {
    if steps == 0 {
        return Vec::new();
    }
    let denom = steps as f32 + 1.0;
    (0..=steps + 1).map(|i| i as f32 / denom).collect()
}

#[derive(Clone, PartialEq)]
struct RangeThumbAccessibility {
    axis: SliderAxis,
    key: &'static str,
    label: Option<String>,
    description: Option<String>,
    fallback_description: &'static str,
    steps: usize,
    disabled: bool,
    value: f32,
    min: f32,
    max: f32,
    value_label: Option<String>,
    keyboard_model: SliderKeyboardModel,
    on_change: CallbackWith<f32>,
}

#[derive(Clone)]
struct RangeSliderThumbProps {
    thumb_layout: SliderLayout,
    handle_width: Px,
    colors: SliderColors,
    focus: FocusRequester,
    accessibility: RangeThumbAccessibility,
}

fn range_thumb_label(args: &RangeSliderConfig, start: bool) -> String {
    let override_label = if start {
        &args.start_accessibility_label
    } else {
        &args.end_accessibility_label
    };
    if let Some(label) = override_label {
        return label.clone();
    }
    let edge = if start { "start" } else { "end" };
    match &args.accessibility_label {
        Some(label) => format!("{label} {edge}"),
        None => format!("Range {edge}"),
    }
}

fn apply_range_thumb_semantics(
    accessibility: &mut AccessibilityNode,
    action_handler: &mut Option<AccessibilityActionHandler>,
    args: &RangeThumbAccessibility,
) {
    accessibility.role = Some(Role::Slider);
    accessibility.orientation = Some(args.axis.accessibility_orientation());
    accessibility.key = Some(args.key.to_string());
    accessibility.label = args.label.clone();
    accessibility.description = Some(
        args.description
            .as_ref()
            .map(|description| format!("{description} ({})", args.fallback_description))
            .unwrap_or_else(|| args.fallback_description.to_string()),
    );
    accessibility.numeric_value = Some(args.value as f64);
    accessibility.value = args.value_label.clone();
    accessibility.min_numeric_value = Some(args.min as f64);
    accessibility.max_numeric_value = Some(args.max as f64);
    accessibility.focusable = !args.disabled;
    accessibility.disabled = args.disabled;
    accessibility.actions.clear();

    if args.disabled {
        *action_handler = None;
        return;
    }

    accessibility.actions.push(Action::Increment);
    accessibility.actions.push(Action::Decrement);

    let delta = if args.steps == 0 {
        ACCESSIBILITY_STEP
    } else {
        1.0 / (args.steps as f32 + 1.0)
    };
    let value = args.value;
    let min = args.min;
    let max = args.max;
    let steps = args.steps;
    let on_change = args.on_change;
    *action_handler = Some(Box::new(move |action| {
        let next = match action {
            Action::Increment => value + delta,
            Action::Decrement => value - delta,
            _ => return,
        };
        let next = snap_fraction(next, steps).clamp(min, max);
        if (next - value).abs() > f32::EPSILON {
            on_change.call(next);
        }
    }));
}

#[tessera]
fn range_slider_thumb(
    thumb_layout: Option<SliderLayout>,
    handle_width: Option<Px>,
    colors: Option<SliderColors>,
    focus: Option<FocusRequester>,
    accessibility: Option<RangeThumbAccessibility>,
    indicator_anchor: Option<State<tessera_ui::LayoutCoordinates>>,
) {
    let thumb_layout = thumb_layout.expect("range_slider_thumb requires thumb layout to be set");
    let handle_width = handle_width.unwrap_or(Px::ZERO);
    let colors = colors.expect("range_slider_thumb requires colors to be set");
    let focus = focus.expect("range_slider_thumb requires focus to be set");
    let accessibility = accessibility.expect("range_slider_thumb requires accessibility to be set");
    #[cfg(test)]
    tests::record_thumb(accessibility.clone());
    let modifier = apply_range_thumb_pointer_modifier(
        Modifier::new()
            .focus_requester(focus)
            .focusable()
            .focus_properties(
                FocusProperties::new()
                    .can_focus(!accessibility.disabled)
                    .can_request_focus(!accessibility.disabled),
            ),
        accessibility,
    );

    let modifier = if let Some(anchor) = indicator_anchor {
        modifier.on_layout_coordinates(move |rect| {
            if anchor.get() != rect {
                anchor.set(rect);
            }
        })
    } else {
        modifier
    };
    layout().modifier(modifier).child(move || {
        slider_handle()
            .layout(thumb_layout)
            .width(handle_width)
            .colors(colors)
            .focused(focus.is_focus_visible());
    });
}

#[derive(Clone)]
struct FocusTargetModifier {
    requester: FocusRequester,
    disabled: bool,
}

impl FocusTargetModifier {
    fn build(self) -> Modifier {
        Modifier::new()
            .focus_requester(self.requester)
            .focusable()
            .focus_properties(
                FocusProperties::new()
                    .can_focus(!self.disabled)
                    .can_request_focus(!self.disabled),
            )
    }
}

struct RangeSliderThumbSemanticsModifierNode {
    accessibility: RangeThumbAccessibility,
}

impl SemanticsModifierNode for RangeSliderThumbSemanticsModifierNode {
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        action_handler: &mut Option<AccessibilityActionHandler>,
    ) {
        apply_range_thumb_semantics(accessibility, action_handler, &self.accessibility);
    }
}

fn apply_range_thumb_pointer_modifier(
    base: Modifier,
    accessibility: RangeThumbAccessibility,
) -> Modifier {
    base.push_keyboard_input(SliderKeyboardModifierNode {
        axis: accessibility.axis,
        model: accessibility.keyboard_model,
        steps: accessibility.steps,
        disabled: accessibility.disabled,
        on_change: accessibility.on_change,
    })
    .push_semantics(RangeSliderThumbSemanticsModifierNode { accessibility })
}

struct SliderPointerModifierNode {
    controller: State<SliderController>,
    args: SliderConfig,
    tap_recognizer: State<TapRecognizer>,
    drag_recognizer: State<DragRecognizer>,
}

impl PointerInputModifierNode for SliderPointerModifierNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        let visual = self
            .controller
            .with(|controller| controller.visual_state(&self.args));
        let base_handle_width = self.args.thumb_diameter.to_px();
        let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
        let handle_width = if visual.is_dragged() || visual.is_pressed() || visual.is_focused() {
            pressed_handle_width
        } else {
            base_handle_width
        };
        let resolved_layout = slider_layout_with_handle_width(
            &self.args,
            self.args.axis.main_length(input.computed_data),
            handle_width,
        );
        handle_slider_state(
            &mut input,
            self.tap_recognizer,
            self.drag_recognizer,
            self.controller,
            &self.args,
            &resolved_layout,
        );
    }
}

struct SliderSemanticsModifierNode {
    args: SliderConfig,
    clamped_value: f32,
}

impl SemanticsModifierNode for SliderSemanticsModifierNode {
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        action_handler: &mut Option<AccessibilityActionHandler>,
    ) {
        apply_slider_semantics(
            accessibility,
            action_handler,
            &self.args,
            self.clamped_value,
            &self.args.on_change,
        );
    }
}

fn apply_slider_pointer_modifier(
    base: Modifier,
    controller: State<SliderController>,
    args: SliderConfig,
    tap_recognizer: State<TapRecognizer>,
    drag_recognizer: State<DragRecognizer>,
    clamped_value: f32,
    keyboard_value: State<f32>,
) -> Modifier {
    let modifier = if !args.disabled {
        base.hover_cursor_icon(tessera_ui::winit::window::CursorIcon::Pointer)
    } else {
        base
    };
    modifier
        .push_keyboard_input(SliderKeyboardModifierNode {
            axis: args.axis,
            model: SliderKeyboardModel::Single(keyboard_value),
            steps: args.steps,
            disabled: args.disabled,
            on_change: args.on_change,
        })
        .push_semantics(SliderSemanticsModifierNode {
            args: args.clone(),
            clamped_value,
        })
        .push_pointer_input(SliderPointerModifierNode {
            controller,
            args,
            tap_recognizer,
            drag_recognizer,
        })
}

struct CenteredSliderPointerModifierNode {
    controller: State<SliderController>,
    args: SliderConfig,
    tap_recognizer: State<TapRecognizer>,
    drag_recognizer: State<DragRecognizer>,
}

impl PointerInputModifierNode for CenteredSliderPointerModifierNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        let visual = self
            .controller
            .with(|controller| controller.visual_state(&self.args));
        let base_handle_width = self.args.thumb_diameter.to_px();
        let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
        let handle_width = if visual.is_dragged() || visual.is_pressed() || visual.is_focused() {
            pressed_handle_width
        } else {
            base_handle_width
        };
        let resolved_layout = CenteredSliderLayout {
            base: slider_layout_with_handle_width(
                &self.args,
                self.args.axis.main_length(input.computed_data),
                handle_width,
            ),
        };
        handle_slider_state(
            &mut input,
            self.tap_recognizer,
            self.drag_recognizer,
            self.controller,
            &self.args,
            &resolved_layout.base,
        );
    }
}

fn apply_centered_slider_pointer_modifier(
    base: Modifier,
    controller: State<SliderController>,
    args: SliderConfig,
    tap_recognizer: State<TapRecognizer>,
    drag_recognizer: State<DragRecognizer>,
    clamped_value: f32,
    keyboard_value: State<f32>,
) -> Modifier {
    let modifier = if !args.disabled {
        base.hover_cursor_icon(tessera_ui::winit::window::CursorIcon::Pointer)
    } else {
        base
    };
    modifier
        .push_keyboard_input(SliderKeyboardModifierNode {
            axis: args.axis,
            model: SliderKeyboardModel::Single(keyboard_value),
            steps: args.steps,
            disabled: args.disabled,
            on_change: args.on_change,
        })
        .push_semantics(SliderSemanticsModifierNode {
            args: args.clone(),
            clamped_value,
        })
        .push_pointer_input(CenteredSliderPointerModifierNode {
            controller,
            args,
            tap_recognizer,
            drag_recognizer,
        })
}

struct RangeSliderPointerModifierNode {
    state: State<RangeSliderController>,
    args: RangeSliderConfig,
    tap_recognizer: State<TapRecognizer>,
    drag_recognizer: State<DragRecognizer>,
}

impl PointerInputModifierNode for RangeSliderPointerModifierNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        let resolved_layout =
            range_slider_layout(&self.args, self.args.axis.main_length(input.computed_data));
        let base_handle_width = self.args.thumb_diameter.to_px();
        let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
        let (start_interacting, end_interacting) = self.state.with(|state| {
            (
                state.is_dragging_start || state.focus_start.is_focused(),
                state.is_dragging_end || state.focus_end.is_focused(),
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
            self.tap_recognizer,
            self.drag_recognizer,
            &self.state,
            &self.args,
            &resolved_layout.base,
            RangeSliderHandleWidths {
                start: start_handle_width,
                end: end_handle_width,
            },
        );
    }
}

struct RangeSliderSemanticsModifierNode {
    args: RangeSliderConfig,
    start_value: f32,
    end_value: f32,
}

impl SemanticsModifierNode for RangeSliderSemanticsModifierNode {
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        action_handler: &mut Option<AccessibilityActionHandler>,
    ) {
        apply_range_slider_semantics(
            accessibility,
            &self.args,
            self.start_value,
            self.end_value,
            &self.args.on_change,
        );
        *action_handler = None;
    }
}

fn apply_range_slider_pointer_modifier(
    base: Modifier,
    state: State<RangeSliderController>,
    args: RangeSliderConfig,
    tap_recognizer: State<TapRecognizer>,
    drag_recognizer: State<DragRecognizer>,
    start_value: f32,
    end_value: f32,
) -> Modifier {
    let modifier = if !args.disabled {
        base.hover_cursor_icon(tessera_ui::winit::window::CursorIcon::Pointer)
    } else {
        base
    };
    modifier
        .push_semantics(RangeSliderSemanticsModifierNode {
            args: args.clone(),
            start_value,
            end_value,
        })
        .push_pointer_input(RangeSliderPointerModifierNode {
            state,
            args,
            tap_recognizer,
            drag_recognizer,
        })
}

struct RangeSliderMeasureArgs {
    start: f32,
    end: f32,
    start_handle_width: Px,
    end_handle_width: Px,
    steps: usize,
}

#[derive(Clone)]
struct Slider {
    args: SliderConfig,
    clamped_value: f32,
    handle_width: Px,
    has_inset_icon: bool,
}

impl PartialEq for Slider {
    fn eq(&self, other: &Self) -> bool {
        self.args.size == other.args.size
            && self.args.axis == other.args.axis
            && self.args.show_stop_indicator == other.args.show_stop_indicator
            && self.args.steps == other.args.steps
            && self.clamped_value == other.clamped_value
            && self.handle_width == other.handle_width
            && self.has_inset_icon == other.has_inset_icon
    }
}

impl tessera_ui::LayoutModifierNode for Slider {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let component_width = resolve_component_width(&self.args, input.parent_constraint());
        let resolved_layout =
            slider_layout_with_handle_width(&self.args, component_width, self.handle_width);
        let mut scope = SliderMeasureScope::new(input, self.args.axis);
        let result = measure_slider(
            &mut scope,
            &mut result,
            resolved_layout,
            self.clamped_value,
            self.has_inset_icon && resolved_layout.icon_size.is_some(),
            self.handle_width,
            self.args.steps,
        )?;
        Ok(scope.finish(result))
    }
}

#[derive(Clone)]
struct CenteredSlider {
    args: SliderConfig,
    clamped_value: f32,
    handle_width: Px,
}

impl PartialEq for CenteredSlider {
    fn eq(&self, other: &Self) -> bool {
        self.args.size == other.args.size
            && self.args.axis == other.args.axis
            && self.args.show_stop_indicator == other.args.show_stop_indicator
            && self.args.steps == other.args.steps
            && self.clamped_value == other.clamped_value
            && self.handle_width == other.handle_width
    }
}

impl tessera_ui::LayoutModifierNode for CenteredSlider {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let component_width = resolve_component_width(&self.args, input.parent_constraint());
        let resolved_layout = CenteredSliderLayout {
            base: slider_layout_with_handle_width(&self.args, component_width, self.handle_width),
        };
        let mut scope = SliderMeasureScope::new(input, self.args.axis);
        let result = measure_centered_slider(
            &mut scope,
            &mut result,
            resolved_layout,
            self.clamped_value,
            self.handle_width,
            self.args.steps,
        )?;
        Ok(scope.finish(result))
    }
}

#[derive(Clone)]
struct RangeSlider {
    args: RangeSliderConfig,
    slider: SliderConfig,
    start: f32,
    end: f32,
    start_handle_width: Px,
    end_handle_width: Px,
}

impl PartialEq for RangeSlider {
    fn eq(&self, other: &Self) -> bool {
        self.args.size == other.args.size
            && self.args.axis == other.args.axis
            && self.args.show_stop_indicator == other.args.show_stop_indicator
            && self.args.steps == other.args.steps
            && self.start == other.start
            && self.end == other.end
            && self.start_handle_width == other.start_handle_width
            && self.end_handle_width == other.end_handle_width
    }
}

impl tessera_ui::LayoutModifierNode for RangeSlider {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let component_width = resolve_component_width(&self.slider, input.parent_constraint());
        let resolved_layout = range_slider_layout(&self.args, component_width);
        let mut scope = SliderMeasureScope::new(input, self.args.axis);
        let result = measure_range_slider(
            &mut scope,
            &mut result,
            resolved_layout,
            RangeSliderMeasureArgs {
                start: self.start,
                end: self.end,
                start_handle_width: self.start_handle_width,
                end_handle_width: self.end_handle_width,
                steps: self.args.steps,
            },
        )?;
        Ok(scope.finish(result))
    }
}

/// Controller for the `slider` component.
pub struct SliderController {
    is_dragging: bool,
    focus: FocusRequester,
    is_hovered: bool,
}

impl SliderController {
    fn visual_state(&self, args: &SliderConfig) -> InteractionState {
        let mut live = InteractionState::new();
        live.set_dragged(self.is_dragging());
        live.set_focused(self.is_focused());
        live.set_hovered(self.is_hovered());
        live.with_presentation(args.presentation_state, !args.disabled)
    }

    /// Creates a new slider controller.
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            focus: FocusRequester::new(),
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
#[derive(Clone, PartialEq)]
struct SliderConfig {
    pub presentation_state: Option<InteractionVisualState>,
    pub value_indicator: SliderValueIndicatorVisibility,
    pub value_label: Option<String>,
    axis: SliderAxis,
    /// Modifier chain applied to the slider subtree.
    pub modifier: Modifier,
    /// The current value of the slider, ranging from 0.0 to 1.0.
    pub value: f32,
    /// Callback function triggered when the slider's value changes.
    pub on_change: CallbackWith<f32>,
    /// Size variant of the slider.
    pub size: SliderSize,
    /// The color of the active part of the track (progress fill).
    pub active_track_color: Color,
    /// The color of the inactive part of the track (background).
    pub inactive_track_color: Color,
    /// Foreground for tick marks on the active track.
    pub active_tick_color: Color,
    /// Foreground for inactive tick marks and stop indicators.
    pub inactive_tick_color: Color,
    /// The thickness of the handle indicator.
    pub thumb_diameter: Dp,
    /// Color of the handle indicator.
    pub thumb_color: Color,
    /// Disable interaction.
    pub disabled: bool,
    /// Optional accessibility label read by assistive technologies.
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
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
    pub inset_icon: Option<Painter>,
    /// Optional external controller for drag and focus state.
    ///
    /// When this is `None`, `slider` and `centered_slider` create and own an
    /// internal controller.
    pub controller: Option<State<SliderController>>,
}

impl Default for SliderConfig {
    fn default() -> Self {
        Self {
            presentation_state: None,
            value_indicator: SliderValueIndicatorVisibility::Hidden,
            value_label: None,
            axis: SliderAxis::default(),
            modifier: Modifier::new(),
            value: 0.0,
            on_change: CallbackWith::default_value(),
            size: SliderSize::default(),
            active_track_color: Color::new(0.0, 0.5, 1.0, 1.0),
            inactive_track_color: Color::new(0.85, 0.9, 0.95, 1.0),
            active_tick_color: Color::WHITE,
            inactive_tick_color: Color::BLACK,
            thumb_diameter: Dp(4.0),
            thumb_color: Color::new(0.0, 0.5, 1.0, 1.0),
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
#[derive(Clone, PartialEq)]
struct RangeSliderConfig {
    pub value_indicator: SliderValueIndicatorVisibility,
    pub start_value_label: Option<String>,
    pub end_value_label: Option<String>,
    axis: SliderAxis,
    /// Modifier chain applied to the range slider subtree.
    pub modifier: Modifier,
    /// The current range values (start, end), each between 0.0 and 1.0.
    pub value: (f32, f32),

    /// Callback function triggered when the range values change.
    pub on_change: CallbackWith<(f32, f32)>,

    /// Size variant of the slider.
    pub size: SliderSize,

    /// The color of the active part of the track (range fill).
    pub active_track_color: Color,

    /// The color of the inactive part of the track (background).
    pub inactive_track_color: Color,
    /// Foreground for tick marks on the active track.
    pub active_tick_color: Color,
    /// Foreground for inactive tick marks and stop indicators.
    pub inactive_tick_color: Color,

    /// The thickness of the handle indicators.
    pub thumb_diameter: Dp,

    /// Color of the handle indicators.
    pub thumb_color: Color,

    /// Disable interaction.
    pub disabled: bool,
    /// Optional accessibility label.
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
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
    pub controller: Option<State<RangeSliderController>>,
    /// Accessible name for the lower handle.
    pub start_accessibility_label: Option<String>,
    /// Accessible name for the upper handle.
    pub end_accessibility_label: Option<String>,
}

impl Default for RangeSliderConfig {
    fn default() -> Self {
        Self {
            value_indicator: SliderValueIndicatorVisibility::Hidden,
            start_value_label: None,
            end_value_label: None,
            axis: SliderAxis::default(),
            modifier: Modifier::new(),
            value: (0.0, 1.0),
            on_change: CallbackWith::default_value(),
            size: SliderSize::default(),
            active_track_color: Color::new(0.0, 0.5, 1.0, 1.0),
            inactive_track_color: Color::new(0.85, 0.9, 0.95, 1.0),
            active_tick_color: Color::WHITE,
            inactive_tick_color: Color::BLACK,
            thumb_diameter: Dp(4.0),
            thumb_color: Color::new(0.0, 0.5, 1.0, 1.0),
            disabled: false,
            accessibility_label: None,
            accessibility_description: None,
            show_stop_indicator: true,
            steps: 0,
            controller: None,
            start_accessibility_label: None,
            end_accessibility_label: None,
        }
    }
}

type SliderArgs = SliderConfig;
type RangeSliderArgs = RangeSliderConfig;

struct SliderParams {
    presentation_state: Option<InteractionVisualState>,
    value_indicator: Option<SliderValueIndicatorVisibility>,
    value_label: Option<String>,
    axis: SliderAxis,
    modifier: Option<Modifier>,
    value: f32,
    on_change: Option<CallbackWith<f32>>,
    size: SliderSize,
    active_track_color: Option<Color>,
    inactive_track_color: Option<Color>,
    active_tick_color: Option<Color>,
    inactive_tick_color: Option<Color>,
    thumb_diameter: Option<Dp>,
    thumb_color: Option<Color>,
    disabled: bool,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    show_stop_indicator: Option<bool>,
    steps: usize,
    inset_icon: Option<Painter>,
    controller: Option<State<SliderController>>,
}

/// Slider colors derived from the active Material color scheme.
#[derive(Clone, Copy)]
struct SliderThemeColors {
    active_track: Color,
    inactive_track: Color,
    active_tick: Color,
    inactive_tick: Color,
    thumb: Color,
}

/// Resolves the default slider colors from the active Material theme.
fn slider_theme_colors(scheme: &MaterialColorScheme) -> SliderThemeColors {
    SliderThemeColors {
        active_track: scheme.primary,
        inactive_track: scheme.secondary_container,
        active_tick: scheme.on_primary,
        inactive_tick: scheme.on_secondary_container,
        thumb: scheme.primary,
    }
}

fn slider_config_from_params(params: SliderParams, theme: SliderThemeColors) -> SliderConfig {
    let defaults = SliderConfig::default();
    SliderConfig {
        presentation_state: params.presentation_state,
        value_indicator: params.value_indicator.unwrap_or_default(),
        value_label: params.value_label,
        axis: params.axis,
        modifier: params.modifier.unwrap_or(defaults.modifier),
        value: snap_fraction(params.value, params.steps),
        on_change: params.on_change.unwrap_or_else(CallbackWith::default_value),
        size: params.size,
        active_track_color: params.active_track_color.unwrap_or(theme.active_track),
        inactive_track_color: params.inactive_track_color.unwrap_or(theme.inactive_track),
        active_tick_color: params.active_tick_color.unwrap_or(theme.active_tick),
        inactive_tick_color: params.inactive_tick_color.unwrap_or(theme.inactive_tick),
        thumb_diameter: params.thumb_diameter.unwrap_or(defaults.thumb_diameter),
        thumb_color: params.thumb_color.unwrap_or(theme.thumb),
        disabled: params.disabled,
        accessibility_label: params.accessibility_label,
        accessibility_description: params.accessibility_description,
        show_stop_indicator: params
            .show_stop_indicator
            .unwrap_or(defaults.show_stop_indicator),
        steps: params.steps,
        inset_icon: params.inset_icon,
        controller: params.controller,
    }
}

struct RangeSliderParams {
    value_indicator: Option<SliderValueIndicatorVisibility>,
    start_value_label: Option<String>,
    end_value_label: Option<String>,
    axis: SliderAxis,
    modifier: Option<Modifier>,
    value: (f32, f32),
    on_change: Option<CallbackWith<(f32, f32)>>,
    size: SliderSize,
    active_track_color: Option<Color>,
    inactive_track_color: Option<Color>,
    active_tick_color: Option<Color>,
    inactive_tick_color: Option<Color>,
    thumb_diameter: Option<Dp>,
    thumb_color: Option<Color>,
    disabled: bool,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    show_stop_indicator: Option<bool>,
    steps: usize,
    controller: Option<State<RangeSliderController>>,
    start_accessibility_label: Option<String>,
    end_accessibility_label: Option<String>,
}

fn range_slider_config_from_params(
    params: RangeSliderParams,
    theme: SliderThemeColors,
) -> RangeSliderConfig {
    let defaults = RangeSliderConfig::default();
    let start = snap_fraction(params.value.0, params.steps);
    let end = snap_fraction(params.value.1, params.steps).max(start);
    RangeSliderConfig {
        value_indicator: params.value_indicator.unwrap_or_default(),
        start_value_label: params.start_value_label,
        end_value_label: params.end_value_label,
        axis: params.axis,
        modifier: params.modifier.unwrap_or(defaults.modifier),
        value: (start, end),
        on_change: params.on_change.unwrap_or_else(CallbackWith::default_value),
        size: params.size,
        active_track_color: params.active_track_color.unwrap_or(theme.active_track),
        inactive_track_color: params.inactive_track_color.unwrap_or(theme.inactive_track),
        active_tick_color: params.active_tick_color.unwrap_or(theme.active_tick),
        inactive_tick_color: params.inactive_tick_color.unwrap_or(theme.inactive_tick),
        thumb_diameter: params.thumb_diameter.unwrap_or(defaults.thumb_diameter),
        thumb_color: params.thumb_color.unwrap_or(theme.thumb),
        disabled: params.disabled,
        accessibility_label: params.accessibility_label,
        accessibility_description: params.accessibility_description,
        show_stop_indicator: params
            .show_stop_indicator
            .unwrap_or(defaults.show_stop_indicator),
        steps: params.steps,
        controller: params.controller,
        start_accessibility_label: params.start_accessibility_label,
        end_accessibility_label: params.end_accessibility_label,
    }
}

fn measure_slider(
    input: &mut SliderMeasureScope<'_, '_>,
    result: &mut LayoutResult,
    layout: SliderLayout,
    clamped_value: f32,
    has_inset_icon: bool,
    handle_width: Px,
    steps: usize,
) -> Result<LayoutResult, MeasurementError> {
    let children = input.children();
    let self_width = layout.component_width;
    let self_height = layout.component_height;

    let active = children[0];
    let inactive = children[1];

    let mut current_index = 2;

    let icon_id = if has_inset_icon {
        let active = children[current_index];
        let inactive = children[current_index + 1];
        current_index += 2;
        Some((active, inactive))
    } else {
        None
    };

    let tick_count = if steps == 0 { 0 } else { steps + 2 };
    let tick_ids = &children[current_index..current_index + tick_count];
    current_index += tick_count;

    let stop_id = if layout.show_stop_indicator {
        let id = children.get(current_index).copied();
        current_index += 1;
        id
    } else {
        None
    };

    let handle = children[current_index];

    let active_width = layout.active_width(clamped_value);
    let inactive_width = layout.inactive_width(clamped_value);

    let active_constraint = Constraint::new(
        AxisConstraint::exact(active_width),
        AxisConstraint::exact(layout.track_height),
    );
    active.measure(input, &active_constraint)?;
    result.place_child(active, PxPosition::new(Px(0), layout.track_y));

    let inactive_constraint = Constraint::new(
        AxisConstraint::exact(inactive_width),
        AxisConstraint::exact(layout.track_height),
    );
    inactive.measure(input, &inactive_constraint)?;
    result.place_child(
        inactive,
        PxPosition::new(
            Px(active_width.0 + layout.handle_gap.0 * 2 + handle_width.0),
            layout.track_y,
        ),
    );

    let handle_constraint = Constraint::new(
        AxisConstraint::exact(handle_width),
        AxisConstraint::exact(layout.handle_height),
    );
    handle.measure(input, &handle_constraint)?;

    let handle_center = layout.handle_center(clamped_value);
    result.place_child(
        handle,
        PxPosition::from_f32_xy(
            handle_center.x - handle_width.to_f32() / 2.0,
            layout.handle_y.0 as f32,
        ),
    );

    if let Some(stop) = stop_id {
        let stop_size = layout.stop_indicator_diameter;
        let stop_constraint = Constraint::new(
            AxisConstraint::exact(stop_size),
            AxisConstraint::exact(stop_size),
        );
        stop.measure(input, &stop_constraint)?;
        let stop_offset = layout.center_child_offset(layout.stop_indicator_diameter);
        let inactive_start = active_width.0 + layout.handle_gap.0 * 2 + handle_width.0;
        let corner = layout.track_corner_radius.to_px();
        let stop_center_x = Px(inactive_start + inactive_width.0 - corner.0);
        if layout.contains_marker((Px(inactive_start), inactive_width), stop_center_x.to_f32()) {
            result.place_child(
                stop,
                PxPosition::new(Px(stop_center_x.0 - stop_offset.0), layout.stop_indicator_y),
            );
        }
    }

    if let Some((active_icon, inactive_icon)) = icon_id
        && let Some(icon_size) = layout.icon_size
    {
        let icon_constraint = Constraint::new(
            AxisConstraint::at_most(icon_size.into()),
            AxisConstraint::at_most(icon_size.into()),
        );
        let icon_measured = active_icon.measure(input, &icon_constraint)?;
        let icon_padding = Dp(8.0).to_px();
        let icon_y = layout.track_y + Px((layout.track_height.0 - icon_measured.height.0) / 2);
        let required_width = icon_measured.width + icon_padding * 2;
        if active_width >= required_width {
            result.place_child(active_icon, PxPosition::new(icon_padding, icon_y));
        } else if inactive_width >= required_width {
            inactive_icon.measure(input, &icon_constraint)?;
            result.place_child(
                inactive_icon,
                PxPosition::new(
                    active_width + layout.handle_gap * 2 + handle_width + icon_padding,
                    icon_y,
                ),
            );
        }
    }

    if steps > 0 {
        let tick_size = layout.stop_indicator_diameter;
        let tick_constraint = Constraint::new(
            AxisConstraint::exact(tick_size),
            AxisConstraint::exact(tick_size),
        );
        let tick_offset = layout.center_child_offset(tick_size);
        let start_x = layout.handle_gap.to_f32() + handle_width.to_f32() / 2.0;
        for (i, tick_id) in tick_ids.iter().copied().enumerate() {
            tick_id.measure(input, &tick_constraint)?;
            let fraction = i as f32 / (steps as f32 + 1.0);
            let tick_center_x = start_x + fraction * layout.track_total_width.to_f32();
            let inactive_start = active_width + layout.handle_gap * 2 + handle_width;
            if (layout.show_stop_indicator && i + 1 == tick_count)
                || ![(Px::ZERO, active_width), (inactive_start, inactive_width)]
                    .into_iter()
                    .any(|segment| layout.contains_marker(segment, tick_center_x.round()))
            {
                continue;
            }
            result.place_child(
                tick_id,
                PxPosition::new(
                    Px(tick_center_x.round() as i32 - tick_offset.0),
                    layout.stop_indicator_y,
                ),
            );
        }
    }

    result.size = ComputedData {
        width: self_width,
        height: self_height,
    };
    Ok(result.clone())
}

#[derive(Clone, PartialEq, Copy)]
struct SliderColors {
    active_track: Color,
    inactive_track: Color,
    active_tick: Color,
    inactive_tick: Color,
    thumb: Color,
}

fn slider_colors(args: &SliderConfig, scheme: &MaterialColorScheme) -> SliderColors {
    if args.disabled {
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
            active_tick: scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTAINER),
            inactive_tick: scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTENT),
            thumb: disabled_thumb,
        };
    }

    SliderColors {
        active_track: args.active_track_color,
        inactive_track: args.inactive_track_color,
        active_tick: args.active_tick_color,
        inactive_tick: args.inactive_tick_color,
        thumb: args.thumb_color,
    }
}

fn range_slider_colors(args: &RangeSliderConfig, scheme: &MaterialColorScheme) -> SliderColors {
    if args.disabled {
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
            active_tick: scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTAINER),
            inactive_tick: scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTENT),
            thumb: disabled_thumb,
        };
    }

    SliderColors {
        active_track: args.active_track_color,
        inactive_track: args.inactive_track_color,
        active_tick: args.active_tick_color,
        inactive_tick: args.inactive_tick_color,
        thumb: args.thumb_color,
    }
}

/// # slider
///
/// Select a fractional value for settings with pointer and keyboard input.
///
/// ## Usage
///
/// Use for settings like volume or brightness, or for any user-adjustable
/// value.
///
/// Arrow keys move one step. Home and End select the allowed bounds, and
/// PageUp and PageDown move up to ten steps. Continuous sliders use 1% keyboard
/// steps.
///
/// Supplied values are clamped to 0–1 and, for discrete sliders, rounded to the
/// nearest tick. Ties round upward and NaN resolves to zero. This normalization
/// does not call `on_change` or modify caller-owned state.
///
/// ## Parameters
///
/// - `modifier` — slider placement and size constraints.
/// - `value`, `on_change` — controlled fraction and change callback.
/// - `orientation`, `reverse_direction` — physical axis and increasing-value
///   direction.
/// - `size`, `thumb_diameter` — track size and handle width.
/// - `active_track_color`, `inactive_track_color`, `thumb_color` — optional
///   colors.
/// - `active_tick_color`, `inactive_tick_color` — optional tick foregrounds;
///   inactive tick color also applies to end stops.
/// - `disabled` — whether pointer, keyboard, and accessibility input are
///   disabled.
/// - `accessibility_label`, `accessibility_description` — accessible control
///   name and details.
/// - `show_stop_indicator`, `steps` — end stops and the number of intermediate
///   steps.
/// - `inset_icon` — optional leading icon for larger size variants.
/// - `controller` — optional shared focus and drag state.
///
/// - `presentation_state` — optional visual interaction without changing focus
///   or gestures.
/// - `value_indicator` — hidden by default; show while focused/dragging or
///   always. Enable inside [`slider_value_indicator_provider`].
/// - `value_label` — optional formatted value for the floating label and screen
///   reader; defaults to a rounded percentage without a percent sign when
///   labels are enabled.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::modifier::ModifierExt as _;
/// use tessera_components::slider::slider;
/// use tessera_ui::{Dp, LayoutResult, Modifier};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// slider()
///     .modifier(Modifier::new().width(Dp(200.0)))
///     .value(0.5)
///     .on_change(|new_value| {
///         assert!((0.0..=1.0).contains(&new_value));
///     });
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn slider(
    presentation_state: Option<InteractionVisualState>,
    modifier: Option<Modifier>,
    value: Option<f32>,
    on_change: Option<CallbackWith<f32>>,
    size: Option<SliderSize>,
    orientation: Option<SliderOrientation>,
    reverse_direction: Option<bool>,
    active_track_color: Option<Color>,
    inactive_track_color: Option<Color>,
    active_tick_color: Option<Color>,
    inactive_tick_color: Option<Color>,
    thumb_diameter: Option<Dp>,
    thumb_color: Option<Color>,
    disabled: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    show_stop_indicator: Option<bool>,
    steps: Option<usize>,
    value_indicator: Option<SliderValueIndicatorVisibility>,
    #[prop(into)] value_label: Option<String>,
    #[prop(into)] inset_icon: Option<Painter>,
    controller: Option<State<SliderController>>,
) {
    let scheme = use_context::<MaterialTheme>()
        .map(|theme| theme.get().color_scheme)
        .unwrap_or_else(|| MaterialTheme::default().color_scheme);
    let defaults = SliderConfig::default();
    let value = value.unwrap_or(defaults.value);
    let size = size.unwrap_or(defaults.size);
    let disabled = disabled.unwrap_or(defaults.disabled);
    let steps = steps.unwrap_or(defaults.steps);
    let args = slider_config_from_params(
        SliderParams {
            presentation_state,
            axis: SliderAxis {
                orientation: orientation.unwrap_or_default(),
                reversed: reverse_direction.unwrap_or(false),
            },
            modifier,
            value,
            on_change,
            size,
            active_track_color,
            inactive_track_color,
            active_tick_color,
            inactive_tick_color,
            thumb_diameter,
            thumb_color,
            disabled,
            accessibility_label,
            accessibility_description,
            show_stop_indicator,
            steps,
            value_indicator,
            value_label,
            inset_icon,
            controller,
        },
        slider_theme_colors(&scheme),
    );
    let controller = if let Some(controller) = args.controller {
        controller
    } else {
        remember(SliderController::new)
    };
    let tap_recognizer = remember(TapRecognizer::default);
    let drag_recognizer = remember(DragRecognizer::default);
    let mut resolved_args = args;
    resolved_args.controller = Some(controller);
    slider_content()
        .args(resolved_args)
        .tap_recognizer(tap_recognizer)
        .drag_recognizer(drag_recognizer);
}

#[tessera]
fn slider_content(
    args: Option<SliderConfig>,
    tap_recognizer: Option<State<TapRecognizer>>,
    drag_recognizer: Option<State<DragRecognizer>>,
) {
    let args = args.expect("slider_content requires args to be set");
    let tap_recognizer = tap_recognizer.expect("slider_content requires tap recognizer to be set");
    let drag_recognizer =
        drag_recognizer.expect("slider_content requires drag recognizer to be set");
    let controller = args
        .controller
        .expect("slider_content requires controller to be set");
    let initial_width = fallback_component_width(&args);
    let clamped_value = args.value.clamp(0.0, 1.0);
    let keyboard_value = remember(|| clamped_value);
    if keyboard_value.get() != clamped_value {
        keyboard_value.set(clamped_value);
    }
    let visual = controller.with(|c| c.visual_state(&args));
    let base_handle_width = args.thumb_diameter.to_px();
    let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
    let handle_width = if visual.is_dragged() || visual.is_pressed() || visual.is_focused() {
        pressed_handle_width
    } else {
        base_handle_width
    };
    let modifier = apply_slider_pointer_modifier(
        args.modifier.clone().then(
            FocusTargetModifier {
                requester: controller.with(|c| c.focus),
                disabled: args.disabled,
            }
            .build(),
        ),
        controller,
        args.clone(),
        tap_recognizer,
        drag_recognizer,
        clamped_value,
        keyboard_value,
    );

    let indicator_anchor = if args.value_indicator != SliderValueIndicatorVisibility::Hidden {
        let anchor = remember(tessera_ui::LayoutCoordinates::default);
        indicator::register_indicator()
            .anchor(anchor)
            .label(
                args.value_label
                    .clone()
                    .unwrap_or_else(|| default_value_label(clamped_value)),
            )
            .visible(args.value_indicator.visible(
                visual.is_dragged() || visual.is_pressed() || visual.is_focused(),
                args.disabled,
            ))
            .orientation(args.axis.orientation)
            .interacting(visual.is_dragged() || visual.is_pressed() || visual.is_focused());
        Some(anchor)
    } else {
        None
    };
    let next = (
        args.clone(),
        initial_width,
        clamped_value,
        handle_width,
        indicator_anchor,
        visual.is_focused()
            && controller
                .with(|controller| !controller.is_focused() || controller.focus.is_focus_visible()),
    );
    let render_data = remember(|| next.clone());
    if render_data.with(|current| current != &next) {
        render_data.set(next);
    }
    layout()
        .modifier(modifier)
        .policy(Slider {
            args: args.clone(),
            clamped_value,
            handle_width,
            has_inset_icon: args.inset_icon.is_some(),
        })
        .child(move || {
            let (args, initial_width, clamped_value, handle_width, indicator_anchor, focused) =
                render_data.get();
            let slider_layout = slider_layout_with_handle_width(&args, initial_width, handle_width);
            let scheme = use_context::<MaterialTheme>()
                .map(|theme| theme.get().color_scheme)
                .unwrap_or_else(|| MaterialTheme::default().color_scheme);
            let colors = slider_colors(&args, &scheme);

            active_segment().layout(slider_layout).colors(colors);
            inactive_segment().layout(slider_layout).colors(colors);

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

                icon()
                    .painter(inset_icon.clone())
                    .tint(tint)
                    .tint_mode(TintMode::Solid)
                    .size(icon_size);
                icon()
                    .painter(inset_icon.clone())
                    .tint(if args.disabled {
                        scheme
                            .on_surface
                            .with_alpha(MaterialAlpha::DISABLED_CONTENT)
                    } else {
                        scheme.on_secondary_container
                    })
                    .tint_mode(TintMode::Solid)
                    .size(icon_size);
            }

            if args.steps > 0 {
                for fraction in tick_fractions(args.steps) {
                    let is_active = fraction <= clamped_value;
                    let color = if is_active {
                        colors.active_tick
                    } else {
                        colors.inactive_tick
                    };
                    slider_tick()
                        .diameter(slider_layout.stop_indicator_diameter)
                        .color(color);
                }
            }
            if slider_layout.show_stop_indicator {
                stop_indicator().layout(slider_layout).colors(colors);
            }
            slider_handle()
                .layout(slider_layout)
                .width(handle_width)
                .colors(colors)
                .focused(focused)
                .indicator_anchor_optional(indicator_anchor);
        });
}

fn measure_centered_slider(
    input: &mut SliderMeasureScope<'_, '_>,
    result: &mut LayoutResult,
    layout: CenteredSliderLayout,
    value: f32,
    handle_width: Px,
    steps: usize,
) -> Result<LayoutResult, MeasurementError> {
    let children = input.children();
    let self_width = layout.base.component_width;
    let self_height = layout.base.component_height;
    let track_y = layout.base.track_y;

    let left_inactive = children[0];
    let active = children[1];
    let right_inactive = children[2];
    let mut current_index = 3;
    let tick_count = if steps == 0 { 0 } else { steps + 2 };
    let tick_ids = &children[current_index..current_index + tick_count];
    current_index += tick_count;

    let (left_stop_id, right_stop_id) = if layout.base.show_stop_indicator {
        let left = children[current_index];
        let right = children[current_index + 1];
        current_index += 2;
        (Some(left), Some(right))
    } else {
        (None, None)
    };
    let handle = children[current_index];

    let segments = layout.segments(value);

    // 1. Left Inactive
    left_inactive.measure(
        input,
        &Constraint::new(
            AxisConstraint::exact(segments.left_inactive.1),
            AxisConstraint::exact(layout.base.track_height),
        ),
    )?;
    result.place_child(
        left_inactive,
        PxPosition::new(segments.left_inactive.0, track_y),
    );

    // 2. Active
    active.measure(
        input,
        &Constraint::new(
            AxisConstraint::exact(segments.active.1),
            AxisConstraint::exact(layout.base.track_height),
        ),
    )?;
    result.place_child(active, PxPosition::new(segments.active.0, track_y));

    // 3. Right Inactive
    right_inactive.measure(
        input,
        &Constraint::new(
            AxisConstraint::exact(segments.right_inactive.1),
            AxisConstraint::exact(layout.base.track_height),
        ),
    )?;
    result.place_child(
        right_inactive,
        PxPosition::new(segments.right_inactive.0, track_y),
    );

    // 4. Handle
    handle.measure(
        input,
        &Constraint::new(
            AxisConstraint::exact(handle_width),
            AxisConstraint::exact(layout.base.handle_height),
        ),
    )?;
    result.place_child(
        handle,
        PxPosition::from_f32_xy(
            segments.handle_center.x - handle_width.to_f32() / 2.0,
            layout.base.handle_y.0 as f32,
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
            AxisConstraint::exact(stop_size),
            AxisConstraint::exact(stop_size),
        );
        left_stop_id.measure(input, &stop_constraint)?;

        let stop_offset = layout.base.center_child_offset(stop_size);
        let stop_padding = layout.stop_indicator_offset();

        let left_stop_x = Px(stop_padding.0);

        if layout
            .base
            .contains_marker(segments.left_inactive, left_stop_x.to_f32())
        {
            result.place_child(
                left_stop_id,
                PxPosition::new(
                    Px(left_stop_x.0 - stop_offset.0),
                    layout.base.stop_indicator_y,
                ),
            );
        }

        // 6. Right Stop
        right_stop_id.measure(input, &stop_constraint)?;
        let right_stop_x = Px(self_width.0 - stop_padding.0);

        if layout
            .base
            .contains_marker(segments.right_inactive, right_stop_x.to_f32())
        {
            result.place_child(
                right_stop_id,
                PxPosition::new(
                    Px(right_stop_x.0 - stop_offset.0),
                    layout.base.stop_indicator_y,
                ),
            );
        }
    }

    if steps > 0 {
        let tick_size = layout.base.stop_indicator_diameter;
        let tick_constraint = Constraint::new(
            AxisConstraint::exact(tick_size),
            AxisConstraint::exact(tick_size),
        );
        let tick_offset = layout.base.center_child_offset(tick_size);
        let start_x = layout.base.handle_gap.to_f32() + handle_width.to_f32() / 2.0;
        for (i, tick_id) in tick_ids.iter().copied().enumerate() {
            tick_id.measure(input, &tick_constraint)?;
            let fraction = i as f32 / (steps as f32 + 1.0);
            let tick_center_x = start_x + fraction * layout.base.track_total_width.to_f32();
            if (layout.base.show_stop_indicator && (i == 0 || i + 1 == tick_count))
                || ![
                    segments.left_inactive,
                    segments.active,
                    segments.right_inactive,
                ]
                .into_iter()
                .any(|segment| layout.base.contains_marker(segment, tick_center_x.round()))
            {
                continue;
            }
            result.place_child(
                tick_id,
                PxPosition::new(
                    Px(tick_center_x.round() as i32 - tick_offset.0),
                    layout.base.stop_indicator_y,
                ),
            );
        }
    }

    result.size = ComputedData {
        width: self_width,
        height: self_height,
    };
    Ok(result.clone())
}

/// # centered_slider
///
/// Adjust a value around a neutral midpoint for balance and deviation settings.
///
/// ## Usage
///
/// Use for adjustments that have a neutral midpoint, such as balance controls
/// or deviation settings.
///
/// Arrow keys move one step. Home and End select the allowed bounds, and
/// PageUp and PageDown move up to ten steps. Continuous sliders use 1% keyboard
/// steps.
///
/// Supplied values are clamped to 0–1 and, for discrete sliders, rounded to the
/// nearest tick. Ties round upward and NaN resolves to zero. This normalization
/// does not call `on_change` or modify caller-owned state.
///
/// ## Parameters
///
/// - `modifier` — slider placement and size constraints.
/// - `value`, `on_change` — controlled fraction and change callback.
/// - `orientation`, `reverse_direction` — physical axis and increasing-value
///   direction.
/// - `size`, `thumb_diameter` — track size and handle width.
/// - `active_track_color`, `inactive_track_color`, `thumb_color` — optional
///   colors.
/// - `active_tick_color`, `inactive_tick_color` — optional tick foregrounds;
///   inactive tick color also applies to end stops.
/// - `disabled` — whether pointer, keyboard, and accessibility input are
///   disabled.
/// - `accessibility_label`, `accessibility_description` — accessible control
///   name and details.
/// - `show_stop_indicator`, `steps` — end stops and the number of intermediate
///   steps.
/// - `inset_icon` — optional leading icon for larger size variants.
/// - `controller` — optional shared focus and drag state.
///
/// - `presentation_state` — optional visual interaction without changing focus
///   or gestures.
/// - `value_indicator` — hidden by default; show while focused/dragging or
///   always. Enable inside [`slider_value_indicator_provider`].
/// - `value_label` — optional formatted value for the floating label and screen
///   reader; defaults to a rounded percentage without a percent sign when
///   labels are enabled.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::modifier::ModifierExt as _;
/// use tessera_components::slider::centered_slider;
/// use tessera_ui::{Dp, Modifier, remember};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// let current_value = remember(|| 0.5f32);
///
/// current_value.set(0.75);
/// assert_eq!(current_value.get(), 0.75);
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(move || {
/// centered_slider()
///     .modifier(Modifier::new().width(Dp(200.0)))
///     .value(current_value.get())
///     .on_change(move |new_value| {
///         current_value.set(new_value);
///     });
///
/// current_value.set(0.25);
/// assert_eq!(current_value.get(), 0.25);
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn centered_slider(
    presentation_state: Option<InteractionVisualState>,
    modifier: Option<Modifier>,
    value: Option<f32>,
    on_change: Option<CallbackWith<f32>>,
    size: Option<SliderSize>,
    orientation: Option<SliderOrientation>,
    reverse_direction: Option<bool>,
    active_track_color: Option<Color>,
    inactive_track_color: Option<Color>,
    active_tick_color: Option<Color>,
    inactive_tick_color: Option<Color>,
    thumb_diameter: Option<Dp>,
    thumb_color: Option<Color>,
    disabled: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    show_stop_indicator: Option<bool>,
    steps: Option<usize>,
    value_indicator: Option<SliderValueIndicatorVisibility>,
    #[prop(into)] value_label: Option<String>,
    #[prop(into)] inset_icon: Option<Painter>,
    controller: Option<State<SliderController>>,
) {
    let scheme = use_context::<MaterialTheme>()
        .map(|theme| theme.get().color_scheme)
        .unwrap_or_else(|| MaterialTheme::default().color_scheme);
    let defaults = SliderConfig::default();
    let value = value.unwrap_or(defaults.value);
    let size = size.unwrap_or(defaults.size);
    let disabled = disabled.unwrap_or(defaults.disabled);
    let steps = steps.unwrap_or(defaults.steps);
    let args = slider_config_from_params(
        SliderParams {
            presentation_state,
            axis: SliderAxis {
                orientation: orientation.unwrap_or_default(),
                reversed: reverse_direction.unwrap_or(false),
            },
            modifier,
            value,
            on_change,
            size,
            active_track_color,
            inactive_track_color,
            active_tick_color,
            inactive_tick_color,
            thumb_diameter,
            thumb_color,
            disabled,
            accessibility_label,
            accessibility_description,
            show_stop_indicator,
            steps,
            value_indicator,
            value_label,
            inset_icon,
            controller,
        },
        slider_theme_colors(&scheme),
    );
    let controller = if let Some(controller) = args.controller {
        controller
    } else {
        remember(SliderController::new)
    };
    let tap_recognizer = remember(TapRecognizer::default);
    let drag_recognizer = remember(DragRecognizer::default);
    let mut resolved_args = args;
    resolved_args.controller = Some(controller);
    centered_slider_content()
        .args(resolved_args)
        .tap_recognizer(tap_recognizer)
        .drag_recognizer(drag_recognizer);
}

#[tessera]
fn centered_slider_content(
    args: Option<SliderConfig>,
    tap_recognizer: Option<State<TapRecognizer>>,
    drag_recognizer: Option<State<DragRecognizer>>,
) {
    let args = args.expect("centered_slider_content requires args to be set");
    let tap_recognizer =
        tap_recognizer.expect("centered_slider_content requires tap recognizer to be set");
    let drag_recognizer =
        drag_recognizer.expect("centered_slider_content requires drag recognizer to be set");
    let controller = args
        .controller
        .expect("centered_slider_content requires controller to be set");
    let initial_width = fallback_component_width(&args);
    let clamped_value = args.value.clamp(0.0, 1.0);
    let keyboard_value = remember(|| clamped_value);
    if keyboard_value.get() != clamped_value {
        keyboard_value.set(clamped_value);
    }
    let visual = controller.with(|c| c.visual_state(&args));
    let base_handle_width = args.thumb_diameter.to_px();
    let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
    let handle_width = if visual.is_dragged() || visual.is_pressed() || visual.is_focused() {
        pressed_handle_width
    } else {
        base_handle_width
    };
    let modifier = apply_centered_slider_pointer_modifier(
        args.modifier.clone().then(
            FocusTargetModifier {
                requester: controller.with(|c| c.focus),
                disabled: args.disabled,
            }
            .build(),
        ),
        controller,
        args.clone(),
        tap_recognizer,
        drag_recognizer,
        clamped_value,
        keyboard_value,
    );

    let indicator_anchor = if args.value_indicator != SliderValueIndicatorVisibility::Hidden {
        let anchor = remember(tessera_ui::LayoutCoordinates::default);
        indicator::register_indicator()
            .anchor(anchor)
            .label(
                args.value_label
                    .clone()
                    .unwrap_or_else(|| default_value_label(clamped_value)),
            )
            .visible(args.value_indicator.visible(
                visual.is_dragged() || visual.is_pressed() || visual.is_focused(),
                args.disabled,
            ))
            .orientation(args.axis.orientation)
            .interacting(visual.is_dragged() || visual.is_pressed() || visual.is_focused());
        Some(anchor)
    } else {
        None
    };
    let next = (
        args.clone(),
        initial_width,
        clamped_value,
        handle_width,
        indicator_anchor,
        visual.is_focused()
            && controller
                .with(|controller| !controller.is_focused() || controller.focus.is_focus_visible()),
    );
    let render_data = remember(|| next.clone());
    if render_data.with(|current| current != &next) {
        render_data.set(next);
    }
    layout()
        .modifier(modifier)
        .policy(CenteredSlider {
            args: args.clone(),
            clamped_value,
            handle_width,
        })
        .child(move || {
            let (args, initial_width, clamped_value, handle_width, indicator_anchor, focused) =
                render_data.get();
            let centered_layout = CenteredSliderLayout {
                base: slider_layout_with_handle_width(&args, initial_width, handle_width),
            };
            let scheme = use_context::<MaterialTheme>()
                .map(|theme| theme.get().color_scheme)
                .unwrap_or_else(|| MaterialTheme::default().color_scheme);
            let colors = slider_colors(&args, &scheme);

            centered_tracks().layout(centered_layout).colors(colors);
            if args.steps > 0 {
                let active_start = clamped_value.min(0.5);
                let active_end = clamped_value.max(0.5);
                for fraction in tick_fractions(args.steps) {
                    let is_active = fraction >= active_start && fraction <= active_end;
                    let color = if is_active {
                        colors.active_tick
                    } else {
                        colors.inactive_tick
                    };
                    slider_tick()
                        .diameter(centered_layout.base.stop_indicator_diameter)
                        .color(color);
                }
            }
            if centered_layout.base.show_stop_indicator {
                centered_stops().layout(centered_layout).colors(colors);
            }
            slider_handle()
                .layout(centered_layout.base)
                .width(handle_width)
                .colors(colors)
                .focused(focused)
                .indicator_anchor_optional(indicator_anchor);
        });
}

fn measure_range_slider(
    input: &mut SliderMeasureScope<'_, '_>,
    result: &mut LayoutResult,
    layout: RangeSliderLayout,
    args: RangeSliderMeasureArgs,
) -> Result<LayoutResult, MeasurementError> {
    let children = input.children();
    let self_width = layout.base.component_width;
    let self_height = layout.base.component_height;
    let track_y = layout.base.track_y;

    let left_inactive = children[0];
    let active = children[1];
    let right_inactive = children[2];
    let mut current_index = 3;
    let tick_count = if args.steps == 0 { 0 } else { args.steps + 2 };
    let tick_ids = &children[current_index..current_index + tick_count];
    current_index += tick_count;

    let (stop_start_id, stop_end_id) = if layout.base.show_stop_indicator {
        let start_id = children.get(current_index).copied();
        let end_id = children.get(current_index + 1).copied();
        current_index += 2;
        (start_id, end_id)
    } else {
        (None, None)
    };

    let handle_start = children[current_index];
    let handle_end = children[current_index + 1];

    let segments = layout.segments(
        args.start,
        args.end,
        args.start_handle_width,
        args.end_handle_width,
    );

    left_inactive.measure(
        input,
        &Constraint::new(
            AxisConstraint::exact(segments.left_inactive.1),
            AxisConstraint::exact(layout.base.track_height),
        ),
    )?;
    result.place_child(
        left_inactive,
        PxPosition::new(segments.left_inactive.0, track_y),
    );

    active.measure(
        input,
        &Constraint::new(
            AxisConstraint::exact(segments.active.1),
            AxisConstraint::exact(layout.base.track_height),
        ),
    )?;
    result.place_child(active, PxPosition::new(segments.active.0, track_y));

    right_inactive.measure(
        input,
        &Constraint::new(
            AxisConstraint::exact(segments.right_inactive.1),
            AxisConstraint::exact(layout.base.track_height),
        ),
    )?;
    result.place_child(
        right_inactive,
        PxPosition::new(segments.right_inactive.0, track_y),
    );

    let start_handle_constraint = Constraint::new(
        AxisConstraint::exact(args.start_handle_width),
        AxisConstraint::exact(layout.base.handle_height),
    );
    let end_handle_constraint = Constraint::new(
        AxisConstraint::exact(args.end_handle_width),
        AxisConstraint::exact(layout.base.handle_height),
    );

    handle_start.measure(input, &start_handle_constraint)?;
    result.place_child(
        handle_start,
        PxPosition::from_f32_xy(
            segments.start_handle_center.x - args.start_handle_width.to_f32() / 2.0,
            layout.base.handle_y.0 as f32,
        ),
    );

    handle_end.measure(input, &end_handle_constraint)?;
    result.place_child(
        handle_end,
        PxPosition::from_f32_xy(
            segments.end_handle_center.x - args.end_handle_width.to_f32() / 2.0,
            layout.base.handle_y.0 as f32,
        ),
    );

    if args.steps > 0 {
        let tick_size = layout.base.stop_indicator_diameter;
        let tick_constraint = Constraint::new(
            AxisConstraint::exact(tick_size),
            AxisConstraint::exact(tick_size),
        );
        let tick_offset = layout.base.center_child_offset(tick_size);

        let component_width = layout.base.component_width.to_f32();
        let gap = layout.base.handle_gap.to_f32();
        let start_half = args.start_handle_width.to_f32() / 2.0;
        let end_half = args.end_handle_width.to_f32() / 2.0;
        let track_total = (component_width - start_half - end_half - gap * 2.0).max(0.0);
        let start_x = gap + start_half;
        for (i, tick_id) in tick_ids.iter().copied().enumerate() {
            tick_id.measure(input, &tick_constraint)?;
            let fraction = i as f32 / (args.steps as f32 + 1.0);
            let tick_center_x = start_x + fraction * track_total;
            if (layout.base.show_stop_indicator && (i == 0 || i + 1 == tick_count))
                || ![
                    segments.left_inactive,
                    segments.active,
                    segments.right_inactive,
                ]
                .into_iter()
                .any(|segment| layout.base.contains_marker(segment, tick_center_x.round()))
            {
                continue;
            }
            result.place_child(
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
            AxisConstraint::exact(stop_size),
            AxisConstraint::exact(stop_size),
        );
        stop_start_id.measure(input, &stop_constraint)?;

        let stop_offset = layout.base.center_child_offset(stop_size);
        let corner = layout.base.track_corner_radius.to_px();
        let start_stop_x = corner;

        if layout
            .base
            .contains_marker(segments.left_inactive, start_stop_x.to_f32())
        {
            result.place_child(
                stop_start_id,
                PxPosition::new(
                    Px(start_stop_x.0 - stop_offset.0),
                    layout.base.stop_indicator_y,
                ),
            );
        }

        stop_end_id.measure(input, &stop_constraint)?;
        let end_stop_x = Px(self_width.0 - corner.0);

        if layout
            .base
            .contains_marker(segments.right_inactive, end_stop_x.to_f32())
        {
            result.place_child(
                stop_end_id,
                PxPosition::new(
                    Px(end_stop_x.0 - stop_offset.0),
                    layout.base.stop_indicator_y,
                ),
            );
        }
    }

    result.size = ComputedData {
        width: self_width,
        height: self_height,
    };
    Ok(result.clone())
}

/// # range_slider
///
/// Select interval endpoints using two independently focusable handles.
///
/// ## Usage
///
/// Use for filtering by range, setting minimum and maximum values, or defining
/// an interval.
///
/// Arrow keys move one step. Home and End select the allowed bounds, and
/// PageUp and PageDown move up to ten steps. Continuous sliders use 1% keyboard
/// steps. Tab keeps a fixed start-then-end handle order; handles cannot cross
/// their neighbor.
///
/// Supplied endpoints are clamped to 0–1 and rounded to the nearest discrete
/// ticks. Ties round upward and NaN resolves to zero. An end below the
/// normalized start is raised to the start. This normalization does not call
/// `on_change` or modify caller-owned state.
///
/// ## Parameters
///
/// - `modifier` — slider placement and size constraints.
/// - `value`, `on_change` — controlled interval and change callback.
/// - `orientation`, `reverse_direction` — physical axis and increasing-value
///   direction.
/// - `size`, `thumb_diameter` — track size and handle width.
/// - `active_track_color`, `inactive_track_color`, `thumb_color` — optional
///   colors.
/// - `active_tick_color`, `inactive_tick_color` — optional tick foregrounds;
///   inactive tick color also applies to end stops.
/// - `disabled` — whether pointer, keyboard, and accessibility input are
///   disabled.
/// - `accessibility_label`, `accessibility_description` — accessible control
///   name and details.
/// - `show_stop_indicator`, `steps` — end stops and the number of intermediate
///   steps.
/// - `controller` — optional shared focus and drag state.
/// - `start_accessibility_label`, `end_accessibility_label` — optional names
///   for each handle.
///
/// - `value_indicator` — hidden by default; show focused/dragged handles or all
///   enabled handles. Enable inside [`slider_value_indicator_provider`].
/// - `start_value_label`, `end_value_label` — optional formatted labels and
///   screen-reader values; defaults are rounded percentages without percent
///   signs.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::modifier::ModifierExt as _;
/// use tessera_components::slider::{SliderOrientation, range_slider};
/// use tessera_ui::{Dp, Modifier, remember};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// let range_value = remember(|| (0.2f32, 0.8f32));
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(move || {
/// range_slider()
///     .modifier(Modifier::new().height(Dp(200.0)))
///     .orientation(SliderOrientation::Vertical)
///     .reverse_direction(true)
///     .value(range_value.get())
///     .on_change(move |(start, end)| {
///         range_value.set((start, end));
///     });
/// assert_eq!(range_value.get(), (0.2, 0.8));
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn range_slider(
    modifier: Option<Modifier>,
    value: Option<(f32, f32)>,
    on_change: Option<CallbackWith<(f32, f32)>>,
    size: Option<SliderSize>,
    orientation: Option<SliderOrientation>,
    reverse_direction: Option<bool>,
    active_track_color: Option<Color>,
    inactive_track_color: Option<Color>,
    active_tick_color: Option<Color>,
    inactive_tick_color: Option<Color>,
    thumb_diameter: Option<Dp>,
    thumb_color: Option<Color>,
    disabled: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    show_stop_indicator: Option<bool>,
    steps: Option<usize>,
    value_indicator: Option<SliderValueIndicatorVisibility>,
    #[prop(into)] start_value_label: Option<String>,
    #[prop(into)] end_value_label: Option<String>,
    controller: Option<State<RangeSliderController>>,
    #[prop(into)] start_accessibility_label: Option<String>,
    #[prop(into)] end_accessibility_label: Option<String>,
) {
    let scheme = use_context::<MaterialTheme>()
        .map(|theme| theme.get().color_scheme)
        .unwrap_or_else(|| MaterialTheme::default().color_scheme);
    let defaults = RangeSliderConfig::default();
    let value = value.unwrap_or(defaults.value);
    let size = size.unwrap_or(defaults.size);
    let disabled = disabled.unwrap_or(defaults.disabled);
    let steps = steps.unwrap_or(defaults.steps);
    let args = range_slider_config_from_params(
        RangeSliderParams {
            axis: SliderAxis {
                orientation: orientation.unwrap_or_default(),
                reversed: reverse_direction.unwrap_or(false),
            },
            modifier,
            value,
            on_change,
            size,
            active_track_color,
            inactive_track_color,
            active_tick_color,
            inactive_tick_color,
            thumb_diameter,
            thumb_color,
            disabled,
            accessibility_label,
            accessibility_description,
            show_stop_indicator,
            steps,
            value_indicator,
            start_value_label,
            end_value_label,
            controller,
            start_accessibility_label,
            end_accessibility_label,
        },
        slider_theme_colors(&scheme),
    );
    let state = if let Some(controller) = args.controller {
        controller
    } else {
        remember(RangeSliderController::new)
    };
    let tap_recognizer = remember(TapRecognizer::default);
    let drag_recognizer = remember(DragRecognizer::default);
    let mut resolved_args = args;
    resolved_args.controller = Some(state);
    range_slider_content()
        .args(resolved_args)
        .tap_recognizer(tap_recognizer)
        .drag_recognizer(drag_recognizer);
}

#[tessera]
fn range_slider_content(
    args: Option<RangeSliderConfig>,
    tap_recognizer: Option<State<TapRecognizer>>,
    drag_recognizer: Option<State<DragRecognizer>>,
) {
    let args = args.expect("range_slider_content requires args to be set");
    let tap_recognizer =
        tap_recognizer.expect("range_slider_content requires tap recognizer to be set");
    let drag_recognizer =
        drag_recognizer.expect("range_slider_content requires drag recognizer to be set");
    let state = args
        .controller
        .expect("range_slider_content requires controller to be set");
    let dummy_slider_args = SliderConfig {
        axis: args.axis,
        size: args.size,
        show_stop_indicator: args.show_stop_indicator,
        ..SliderConfig::default()
    };
    let initial_width = fallback_component_width(&dummy_slider_args);
    let slider = dummy_slider_args.clone();
    let range_layout = range_slider_layout(&args, initial_width);

    let start = args.value.0.clamp(0.0, 1.0);
    let end = args.value.1.clamp(start, 1.0);
    let keyboard_values = remember(|| (start, end));
    if keyboard_values.get() != (start, end) {
        keyboard_values.set((start, end));
    }

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

    let modifier = apply_range_slider_pointer_modifier(
        args.modifier.clone(),
        state,
        args.clone(),
        tap_recognizer,
        drag_recognizer,
        start,
        end,
    );

    let indicator_anchors = if args.value_indicator != SliderValueIndicatorVisibility::Hidden {
        let start_anchor = remember(tessera_ui::LayoutCoordinates::default);
        let end_anchor = remember(tessera_ui::LayoutCoordinates::default);
        indicator::register_indicator()
            .anchor(start_anchor)
            .label(
                args.start_value_label
                    .clone()
                    .unwrap_or_else(|| default_value_label(start)),
            )
            .visible(
                args.value_indicator
                    .visible(start_interacting, args.disabled),
            )
            .orientation(args.axis.orientation)
            .interacting(start_interacting);
        indicator::register_indicator()
            .anchor(end_anchor)
            .label(
                args.end_value_label
                    .clone()
                    .unwrap_or_else(|| default_value_label(end)),
            )
            .visible(args.value_indicator.visible(end_interacting, args.disabled))
            .orientation(args.axis.orientation)
            .interacting(end_interacting);
        [Some(start_anchor), Some(end_anchor)]
    } else {
        [None, None]
    };
    let next = (
        args.clone(),
        range_layout,
        start,
        end,
        start_handle_width,
        end_handle_width,
        indicator_anchors,
    );
    let render_data = remember(|| next.clone());
    if render_data.with(|current| current != &next) {
        render_data.set(next);
    }
    layout()
        .modifier(modifier)
        .policy(RangeSlider {
            args: args.clone(),
            slider,
            start,
            end,
            start_handle_width,
            end_handle_width,
        })
        .child(move || {
            let (
                args,
                range_layout,
                start,
                end,
                start_handle_width,
                end_handle_width,
                indicator_anchors,
            ) = render_data.get();
            let state = args.controller.expect("range slider controller");
            let scheme = use_context::<MaterialTheme>()
                .map(|theme| theme.get().color_scheme)
                .unwrap_or_else(|| MaterialTheme::default().color_scheme);
            let colors = range_slider_colors(&args, &scheme);

            range_tracks().layout(range_layout).colors(colors);
            if args.steps > 0 {
                for fraction in tick_fractions(args.steps) {
                    let is_active = fraction >= start && fraction <= end;
                    let color = if is_active {
                        colors.active_tick
                    } else {
                        colors.inactive_tick
                    };
                    slider_tick()
                        .diameter(range_layout.base.stop_indicator_diameter)
                        .color(color);
                }
            }
            if range_layout.base.show_stop_indicator {
                range_stops().layout(range_layout).colors(colors);
            }

            let start_thumb_args = RangeSliderThumbProps {
                thumb_layout: range_layout.base,
                handle_width: start_handle_width,
                colors,
                focus: state.with(|s| s.focus_start),
                accessibility: RangeThumbAccessibility {
                    key: "range_slider_start_thumb",
                    value_label: args.start_value_label.clone().or_else(|| {
                        (args.value_indicator != SliderValueIndicatorVisibility::Hidden)
                            .then(|| format!("{}%", default_value_label(start)))
                    }),
                    axis: args.axis,
                    label: Some(range_thumb_label(&args, true)),
                    description: args.accessibility_description.clone(),
                    fallback_description: "range start",
                    steps: args.steps,
                    disabled: args.disabled,
                    value: start,
                    min: 0.0,
                    max: end,
                    keyboard_model: SliderKeyboardModel::Start(keyboard_values),
                    on_change: CallbackWith::new(move |new_start: f32| {
                        let on_change = render_data.with(|(args, ..)| args.on_change);
                        if keyboard_values.get().0 != new_start {
                            keyboard_values.with_mut(|value| value.0 = new_start.min(value.1));
                        }
                        on_change.call(keyboard_values.get());
                    }),
                },
            };
            range_slider_thumb()
                .thumb_layout(start_thumb_args.thumb_layout)
                .handle_width(start_thumb_args.handle_width)
                .colors(start_thumb_args.colors)
                .focus(start_thumb_args.focus)
                .accessibility(start_thumb_args.accessibility)
                .indicator_anchor_optional(indicator_anchors[0]);

            let end_thumb_args = RangeSliderThumbProps {
                thumb_layout: range_layout.base,
                handle_width: end_handle_width,
                colors,
                focus: state.with(|s| s.focus_end),
                accessibility: RangeThumbAccessibility {
                    key: "range_slider_end_thumb",
                    value_label: args.end_value_label.clone().or_else(|| {
                        (args.value_indicator != SliderValueIndicatorVisibility::Hidden)
                            .then(|| format!("{}%", default_value_label(end)))
                    }),
                    axis: args.axis,
                    label: Some(range_thumb_label(&args, false)),
                    description: args.accessibility_description.clone(),
                    fallback_description: "range end",
                    steps: args.steps,
                    disabled: args.disabled,
                    value: end,
                    min: start,
                    max: 1.0,
                    keyboard_model: SliderKeyboardModel::End(keyboard_values),
                    on_change: CallbackWith::new(move |new_end: f32| {
                        let on_change = render_data.with(|(args, ..)| args.on_change);
                        if keyboard_values.get().1 != new_end {
                            keyboard_values.with_mut(|value| value.1 = new_end.max(value.0));
                        }
                        on_change.call(keyboard_values.get());
                    }),
                },
            };
            range_slider_thumb()
                .thumb_layout(end_thumb_args.thumb_layout)
                .handle_width(end_thumb_args.handle_width)
                .colors(end_thumb_args.colors)
                .focus(end_thumb_args.focus)
                .accessibility(end_thumb_args.accessibility)
                .indicator_anchor_optional(indicator_anchors[1]);
        });
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer,
        testing::{layout_test, pump_layout_test_session, start_layout_test_session},
    };

    use crate::{column::column, modifier::ModifierExt as _, theme::material_theme};

    use super::*;

    #[derive(Clone, Default)]
    struct Settings {
        steps: usize,
        disabled: bool,
        width: Option<f64>,
    }

    thread_local! {
        static RANGE: Cell<Option<State<(f32, f32)>>> = const { Cell::new(None) };
        static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
        static THUMBS: RefCell<[Option<RangeThumbAccessibility>; 2]> = const { RefCell::new([None, None]) };
    }

    pub(super) fn record_thumb(thumb: RangeThumbAccessibility) {
        let index = usize::from(thumb.key == "range_slider_end_thumb");
        THUMBS.with(|thumbs| thumbs.borrow_mut()[index] = Some(thumb));
    }

    fn thumb(index: usize) -> RangeThumbAccessibility {
        THUMBS.with(|thumbs| thumbs.borrow()[index].clone().expect("range handle"))
    }

    struct SliderTestTag(&'static str);

    impl SemanticsModifierNode for SliderTestTag {
        fn apply(
            &self,
            accessibility: &mut AccessibilityNode,
            _handler: &mut Option<AccessibilityActionHandler>,
        ) {
            accessibility.key = Some(self.0.to_string());
        }
    }

    #[tessera]
    fn fixture() {
        let settings = remember(Settings::default);
        let range = remember(|| (0.2_f32, 0.8_f32));
        let single = remember(|| 0.5_f32);
        let centered = remember(|| 0.5_f32);
        SETTINGS.set(Some(settings));
        RANGE.set(Some(range));
        material_theme().child(move || {
            column().children(move || {
                slider()
                    .modifier(
                        Modifier::new()
                            .width(Dp(settings.with(|settings| settings.width.unwrap_or(280.0))))
                            .push_semantics(SliderTestTag("slider")),
                    )
                    .value(single.get())
                    .on_change(move |value| single.set(value))
                    .steps(settings.with(|settings| settings.steps))
                    .disabled(settings.with(|settings| settings.disabled))
                    .accessibility_label("Volume");
                centered_slider()
                    .modifier(
                        Modifier::new()
                            .width(Dp(settings.with(|settings| settings.width.unwrap_or(280.0))))
                            .push_semantics(SliderTestTag("centered_slider")),
                    )
                    .value(centered.get())
                    .on_change(move |value| centered.set(value))
                    .steps(settings.with(|settings| settings.steps))
                    .disabled(settings.with(|settings| settings.disabled))
                    .accessibility_label("Balance");
                range_slider()
                    .modifier(
                        Modifier::new()
                            .width(Dp(settings.with(|settings| settings.width.unwrap_or(280.0))))
                            .push_semantics(SliderTestTag("range_slider")),
                    )
                    .value(range.get())
                    .on_change(move |value| range.set(value))
                    .steps(settings.with(|settings| settings.steps))
                    .disabled(settings.with(|settings| settings.disabled))
                    .accessibility_label("Price interval")
                    .start_accessibility_label("Minimum price")
                    .end_accessibility_label("Maximum price");
            });
        });
    }

    #[test]
    fn sliders_survive_zero_width_and_restore_their_values_and_bounds() {
        let mut session = start_layout_test_session(layout_test(|| {
            fixture();
        }));
        pump_layout_test_session(&mut session, 0);
        for (frame, width) in [0.0, 1.0, 3.0, 5.0, 280.0].into_iter().enumerate() {
            SETTINGS
                .get()
                .unwrap()
                .with_mut(|settings| settings.width = Some(width));
            let snapshot = pump_layout_test_session(&mut session, (frame as u64 + 1) * 16_666_667);
            for selector in ["slider", "centered_slider", "range_slider"] {
                assert_eq!(snapshot.node(selector).bounds().width, Dp(width).to_px());
            }
            assert_eq!(RANGE.get().unwrap().get(), (0.2, 0.8));
        }
    }

    #[test]
    fn retained_range_callbacks_preserve_the_latest_other_endpoint() {
        let mut session = start_layout_test_session(layout_test(|| {
            fixture();
        }));
        pump_layout_test_session(&mut session, 0);
        let start = thumb(0).on_change;
        let end = thumb(1).on_change;
        RANGE.get().unwrap().set((0.3, 0.6));
        pump_layout_test_session(&mut session, 16_666_667);
        start.call(0.4);
        assert_eq!(RANGE.get().unwrap().get(), (0.4, 0.6));
        pump_layout_test_session(&mut session, 33_333_334);
        end.call(0.9);
        assert_eq!(RANGE.get().unwrap().get(), (0.4, 0.9));
        pump_layout_test_session(&mut session, 50_000_001);
        assert_eq!(thumb(0).max, 0.9);
        assert_eq!(thumb(1).min, 0.4);
    }

    #[test]
    fn changing_steps_rebuilds_ticks_and_disabling_removes_thumb_actions() {
        let mut session = start_layout_test_session(layout_test(|| {
            fixture();
        }));
        pump_layout_test_session(&mut session, 0);
        SETTINGS
            .get()
            .unwrap()
            .with_mut(|settings| settings.steps = 3);
        pump_layout_test_session(&mut session, 16_666_667);
        assert_eq!(thumb(0).steps, 3);
        assert_eq!(thumb(1).steps, 3);
        SETTINGS.get().unwrap().with_mut(|settings| {
            settings.steps = 0;
            settings.disabled = true;
        });
        pump_layout_test_session(&mut session, 33_333_334);
        for index in 0..2 {
            let mut semantics = AccessibilityNode::default();
            let mut handler = None;
            apply_range_thumb_semantics(&mut semantics, &mut handler, &thumb(index));
            assert!(semantics.disabled);
            assert!(!semantics.focusable);
            assert!(semantics.actions.is_empty());
            assert!(handler.is_none());
        }
    }

    fn check_centered_slider_gaps(samples: u32) {
        let mut renderer = HeadlessRenderer::new(
            Box::new(|| {
                fixture();
            }),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 400,
                height: 300,
                sample_count: samples,
                ..Default::default()
            },
        )
        .unwrap();
        for frames in [2, 12] {
            renderer.render_frames(frames);
            let pixels = renderer.read_pixels().unwrap();
            let pixel = |x: usize, y: usize| &pixels[(y * 400 + x) * 4..(y * 400 + x + 1) * 4];
            let background = pixel(300, 72);
            for y in 65..79 {
                for (left, right) in [(135, 144), (136, 143)] {
                    assert_eq!(
                        pixel(left, y),
                        background,
                        "left gap, MSAA {samples}, {frames} frames"
                    );
                    assert_eq!(
                        pixel(right, y),
                        background,
                        "right gap, MSAA {samples}, {frames} frames"
                    );
                }
                assert_ne!(pixel(140, y), background, "vertical handle remains visible");
            }
        }
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn centered_slider_gaps_before_and_after_caching_msaa1() {
        check_centered_slider_gaps(1);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn centered_slider_gaps_before_and_after_caching_msaa4() {
        check_centered_slider_gaps(4);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn published_slider_semantics_keep_distinct_handles_and_current_limits() {
        fn control<'a>(
            tree: &'a tessera_ui::accesskit::TreeUpdate,
            label: &str,
        ) -> (
            tessera_ui::accesskit::NodeId,
            &'a tessera_ui::accesskit::Node,
        ) {
            let (id, node) = tree
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some(label))
                .expect("named control");
            (*id, node)
        }
        let mut renderer = HeadlessRenderer::new(
            Box::new(|| {
                fixture();
            }),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 400,
                height: 300,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(2);
        let before = renderer.accessibility_snapshot().unwrap();
        assert_eq!(
            before
                .nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::Slider)
                .count(),
            4
        );
        let (start_id, start) = control(&before, "Minimum price");
        let (end_id, end) = control(&before, "Maximum price");
        assert_eq!(start.max_numeric_value(), Some(0.8_f32 as f64));
        assert_eq!(end.min_numeric_value(), Some(0.2_f32 as f64));
        let (_, group) = control(&before, "Price interval");
        assert_eq!(group.role(), Role::Group);
        assert!(!group.is_hidden());
        assert!(group.children().contains(&start_id) && group.children().contains(&end_id));
        RANGE.get().unwrap().set((0.3, 0.6));
        renderer.render_frames(2);
        let after = renderer.accessibility_snapshot().unwrap();
        let (id, start) = control(&after, "Minimum price");
        assert_eq!(id, start_id);
        assert_eq!(start.max_numeric_value(), Some(0.6_f32 as f64));
        let (id, end) = control(&after, "Maximum price");
        assert_eq!(id, end_id);
        assert_eq!(end.min_numeric_value(), Some(0.3_f32 as f64));
        SETTINGS
            .get()
            .unwrap()
            .with_mut(|settings| settings.steps = 3);
        renderer.render_frames(2);
        let stepped = renderer.accessibility_snapshot().unwrap();
        assert_eq!(control(&stepped, "Minimum price").0, start_id);
        assert_eq!(control(&stepped, "Maximum price").0, end_id);
        assert_eq!(
            control(&stepped, "Price interval").1.children(),
            &[start_id, end_id]
        );
        SETTINGS
            .get()
            .unwrap()
            .with_mut(|settings| settings.disabled = true);
        renderer.render_frames(2);
        let disabled = renderer.accessibility_snapshot().unwrap();
        for (_, node) in disabled
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Slider)
        {
            assert!(node.is_disabled());
            assert!(!node.supports_action(Action::Focus));
            assert!(!node.supports_action(Action::Increment));
        }
    }
}
