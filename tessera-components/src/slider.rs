//! An interactive slider component for selecting a value in a range.
//!
//! ## Usage
//!
//! Use to allow users to select a value from a continuous range.
use tessera_foundation::gesture::{DragRecognizer, TapRecognizer};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, CallbackWith, Color,
    ComputedData, Constraint, Dp, FocusProperties, FocusRequester, MeasurementError, Modifier,
    PointerInput, PointerInputModifierNode, Px, PxPosition, SemanticsModifierNode, State,
    accesskit::{Action, Role},
    layout::{LayoutInput, LayoutOutput, LayoutPolicy, layout},
    modifier::{CursorModifierExt as _, FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
};

use crate::{
    icon::{IconContent, icon},
    image_vector::TintMode,
    theme::{MaterialAlpha, MaterialTheme},
};

use interaction::{
    RangeSliderHandleWidths, apply_range_slider_semantics, apply_slider_semantics,
    handle_range_slider_state, handle_slider_state, snap_fraction,
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
struct RangeThumbAccessibility {
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

#[derive(Clone)]
struct RangeSliderThumbProps {
    thumb_layout: SliderLayout,
    handle_width: Px,
    colors: SliderColors,
    focus: FocusRequester,
    accessibility: RangeThumbAccessibility,
}

fn apply_range_thumb_semantics(
    accessibility: &mut AccessibilityNode,
    action_handler: &mut Option<AccessibilityActionHandler>,
    args: &RangeThumbAccessibility,
) {
    accessibility.role = Some(Role::Slider);
    accessibility.key = Some(args.key.to_string());
    accessibility.label = args.label.clone();
    accessibility.description = Some(
        args.description
            .as_ref()
            .map(|description| format!("{description} ({})", args.fallback_description))
            .unwrap_or_else(|| args.fallback_description.to_string()),
    );
    accessibility.numeric_value = Some(args.value as f64);
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
        on_change.call(next);
    }));
}

#[tessera]
fn range_slider_thumb(
    thumb_layout: Option<SliderLayout>,
    handle_width: Px,
    colors: Option<SliderColors>,
    focus: Option<FocusRequester>,
    accessibility: Option<RangeThumbAccessibility>,
) {
    let thumb_layout = thumb_layout.expect("range_slider_thumb requires thumb layout to be set");
    let colors = colors.expect("range_slider_thumb requires colors to be set");
    let focus = focus.expect("range_slider_thumb requires focus to be set");
    let accessibility = accessibility.expect("range_slider_thumb requires accessibility to be set");
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

    layout()
        .modifier(modifier)
        .child(move || render_handle(thumb_layout, handle_width, &colors));
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
    base.push_semantics(RangeSliderThumbSemanticsModifierNode { accessibility })
}

struct SliderPointerModifierNode {
    controller: State<SliderController>,
    args: SliderConfig,
    tap_recognizer: State<TapRecognizer>,
    drag_recognizer: State<DragRecognizer>,
}

impl PointerInputModifierNode for SliderPointerModifierNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        let (is_dragging, is_focused) = self
            .controller
            .with(|controller| (controller.is_dragging(), controller.is_focused()));
        let base_handle_width = self.args.thumb_diameter.to_px();
        let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
        let handle_width = if is_dragging || is_focused {
            pressed_handle_width
        } else {
            base_handle_width
        };
        let resolved_layout =
            slider_layout_with_handle_width(&self.args, input.computed_data.width, handle_width);
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
) -> Modifier {
    let modifier = if !args.disabled {
        base.hover_cursor_icon(tessera_ui::winit::window::CursorIcon::Pointer)
    } else {
        base
    };
    modifier
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
        let (is_dragging, is_focused) = self
            .controller
            .with(|controller| (controller.is_dragging(), controller.is_focused()));
        let base_handle_width = self.args.thumb_diameter.to_px();
        let pressed_handle_width = Px((base_handle_width.0 / 2).max(1));
        let handle_width = if is_dragging || is_focused {
            pressed_handle_width
        } else {
            base_handle_width
        };
        let resolved_layout = CenteredSliderLayout {
            base: slider_layout_with_handle_width(
                &self.args,
                input.computed_data.width,
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
) -> Modifier {
    let modifier = if !args.disabled {
        base.hover_cursor_icon(tessera_ui::winit::window::CursorIcon::Pointer)
    } else {
        base
    };
    modifier
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
        let resolved_layout = range_slider_layout(&self.args, input.computed_data.width);
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
struct SliderLayoutPolicy {
    args: SliderConfig,
    clamped_value: f32,
    handle_width: Px,
    has_inset_icon: bool,
}

impl PartialEq for SliderLayoutPolicy {
    fn eq(&self, other: &Self) -> bool {
        self.args.size == other.args.size
            && self.args.show_stop_indicator == other.args.show_stop_indicator
            && self.args.steps == other.args.steps
            && self.clamped_value == other.clamped_value
            && self.handle_width == other.handle_width
            && self.has_inset_icon == other.has_inset_icon
    }
}

impl LayoutPolicy for SliderLayoutPolicy {
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
struct CenteredSliderLayoutPolicy {
    args: SliderConfig,
    clamped_value: f32,
    handle_width: Px,
}

impl PartialEq for CenteredSliderLayoutPolicy {
    fn eq(&self, other: &Self) -> bool {
        self.args.size == other.args.size
            && self.args.show_stop_indicator == other.args.show_stop_indicator
            && self.args.steps == other.args.steps
            && self.clamped_value == other.clamped_value
            && self.handle_width == other.handle_width
    }
}

impl LayoutPolicy for CenteredSliderLayoutPolicy {
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
struct RangeSliderLayoutPolicy {
    args: RangeSliderConfig,
    slider: SliderConfig,
    start: f32,
    end: f32,
    start_handle_width: Px,
    end_handle_width: Px,
}

impl PartialEq for RangeSliderLayoutPolicy {
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

impl LayoutPolicy for RangeSliderLayoutPolicy {
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
    focus: FocusRequester,
    is_hovered: bool,
}

impl SliderController {
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
    pub inset_icon: Option<IconContent>,
    /// Optional external controller for drag and focus state.
    ///
    /// When this is `None`, `slider` and `centered_slider` create and own an
    /// internal controller.
    pub controller: Option<State<SliderController>>,
}

impl Default for SliderConfig {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        Self {
            modifier: Modifier::new(),
            value: 0.0,
            on_change: CallbackWith::default_value(),
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
#[derive(Clone, PartialEq)]
struct RangeSliderConfig {
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
}

impl Default for RangeSliderConfig {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        Self {
            modifier: Modifier::new(),
            value: (0.0, 1.0),
            on_change: CallbackWith::default_value(),
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

type SliderArgs = SliderConfig;
type RangeSliderArgs = RangeSliderConfig;

struct SliderParams {
    modifier: Option<Modifier>,
    value: f32,
    on_change: Option<CallbackWith<f32>>,
    size: SliderSize,
    active_track_color: Option<Color>,
    inactive_track_color: Option<Color>,
    thumb_diameter: Option<Dp>,
    thumb_color: Option<Color>,
    disabled: bool,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    show_stop_indicator: Option<bool>,
    steps: usize,
    inset_icon: Option<IconContent>,
    controller: Option<State<SliderController>>,
}

fn slider_config_from_params(params: SliderParams) -> SliderConfig {
    let defaults = SliderConfig::default();
    SliderConfig {
        modifier: params.modifier.unwrap_or(defaults.modifier),
        value: params.value,
        on_change: params.on_change.unwrap_or_else(CallbackWith::default_value),
        size: params.size,
        active_track_color: params
            .active_track_color
            .unwrap_or(defaults.active_track_color),
        inactive_track_color: params
            .inactive_track_color
            .unwrap_or(defaults.inactive_track_color),
        thumb_diameter: params.thumb_diameter.unwrap_or(defaults.thumb_diameter),
        thumb_color: params.thumb_color.unwrap_or(defaults.thumb_color),
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
    modifier: Option<Modifier>,
    value: (f32, f32),
    on_change: Option<CallbackWith<(f32, f32)>>,
    size: SliderSize,
    active_track_color: Option<Color>,
    inactive_track_color: Option<Color>,
    thumb_diameter: Option<Dp>,
    thumb_color: Option<Color>,
    disabled: bool,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    show_stop_indicator: Option<bool>,
    steps: usize,
    controller: Option<State<RangeSliderController>>,
}

fn range_slider_config_from_params(params: RangeSliderParams) -> RangeSliderConfig {
    let defaults = RangeSliderConfig::default();
    RangeSliderConfig {
        modifier: params.modifier.unwrap_or(defaults.modifier),
        value: params.value,
        on_change: params.on_change.unwrap_or_else(CallbackWith::default_value),
        size: params.size,
        active_track_color: params
            .active_track_color
            .unwrap_or(defaults.active_track_color),
        inactive_track_color: params
            .inactive_track_color
            .unwrap_or(defaults.inactive_track_color),
        thumb_diameter: params.thumb_diameter.unwrap_or(defaults.thumb_diameter),
        thumb_color: params.thumb_color.unwrap_or(defaults.thumb_color),
        disabled: params.disabled,
        accessibility_label: params.accessibility_label,
        accessibility_description: params.accessibility_description,
        show_stop_indicator: params
            .show_stop_indicator
            .unwrap_or(defaults.show_stop_indicator),
        steps: params.steps,
        controller: params.controller,
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
        AxisConstraint::exact(active_width),
        AxisConstraint::exact(layout.track_height),
    );
    input.measure_child(active_id, &active_constraint)?;
    output.place_child(active_id, PxPosition::new(Px(0), layout.track_y));

    let inactive_constraint = Constraint::new(
        AxisConstraint::exact(inactive_width),
        AxisConstraint::exact(layout.track_height),
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
        AxisConstraint::exact(handle_width),
        AxisConstraint::exact(layout.handle_height),
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
            AxisConstraint::exact(stop_size),
            AxisConstraint::exact(stop_size),
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
            AxisConstraint::at_most(icon_size.into()),
            AxisConstraint::at_most(icon_size.into()),
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
            AxisConstraint::exact(tick_size),
            AxisConstraint::exact(tick_size),
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

fn slider_colors(args: &SliderConfig) -> SliderColors {
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

fn range_slider_colors(args: &RangeSliderConfig) -> SliderColors {
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
///   [`SliderConfig`].
/// - `controller` — optional; use [`slider`] to provide your own controller.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::modifier::ModifierExt as _;
/// use tessera_components::slider::slider;
/// use tessera_ui::{Dp, Modifier};
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
    modifier: Option<Modifier>,
    value: f32,
    on_change: Option<CallbackWith<f32>>,
    size: SliderSize,
    active_track_color: Option<Color>,
    inactive_track_color: Option<Color>,
    thumb_diameter: Option<Dp>,
    thumb_color: Option<Color>,
    disabled: bool,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    show_stop_indicator: Option<bool>,
    steps: usize,
    #[prop(into)] inset_icon: Option<IconContent>,
    controller: Option<State<SliderController>>,
) {
    let args = slider_config_from_params(SliderParams {
        modifier,
        value,
        on_change,
        size,
        active_track_color,
        inactive_track_color,
        thumb_diameter,
        thumb_color,
        disabled,
        accessibility_label,
        accessibility_description,
        show_stop_indicator,
        steps,
        inset_icon,
        controller,
    });
    let controller = args
        .controller
        .unwrap_or_else(|| remember(SliderController::new));
    let mut resolved_args = args;
    resolved_args.controller = Some(controller);
    render_slider(resolved_args);
}

fn render_slider(args: SliderConfig) {
    let controller = args
        .controller
        .expect("render_slider requires controller to be set");
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
    let tap_recognizer = remember(TapRecognizer::default);
    let drag_recognizer = remember(DragRecognizer::default);
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
    );

    layout()
        .modifier(modifier)
        .layout_policy(SliderLayoutPolicy {
            args: args.clone(),
            clamped_value,
            handle_width,
            has_inset_icon: args.inset_icon.is_some(),
        })
        .child(move || {
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

                match inset_icon.clone() {
                    IconContent::Vector(data) => {
                        icon()
                            .vector(data)
                            .tint(tint)
                            .tint_mode(TintMode::Solid)
                            .size(icon_size);
                    }
                    IconContent::Raster(data) => {
                        icon().raster(data).size(icon_size);
                    }
                }
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
            AxisConstraint::exact(segments.left_inactive.1),
            AxisConstraint::exact(layout.base.track_height),
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
            AxisConstraint::exact(segments.active.1),
            AxisConstraint::exact(layout.base.track_height),
        ),
    )?;
    output.place_child(active_id, PxPosition::new(segments.active.0, track_y));

    // 3. Right Inactive
    input.measure_child(
        right_inactive_id,
        &Constraint::new(
            AxisConstraint::exact(segments.right_inactive.1),
            AxisConstraint::exact(layout.base.track_height),
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
            AxisConstraint::exact(handle_width),
            AxisConstraint::exact(layout.base.handle_height),
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
            AxisConstraint::exact(stop_size),
            AxisConstraint::exact(stop_size),
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
            AxisConstraint::exact(tick_size),
            AxisConstraint::exact(tick_size),
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
///   [`SliderConfig`].
/// - `controller` — optional controller; use [`centered_slider`] to supply one.
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
/// // Simulate a value change
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
    modifier: Option<Modifier>,
    value: f32,
    on_change: Option<CallbackWith<f32>>,
    size: SliderSize,
    active_track_color: Option<Color>,
    inactive_track_color: Option<Color>,
    thumb_diameter: Option<Dp>,
    thumb_color: Option<Color>,
    disabled: bool,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    show_stop_indicator: Option<bool>,
    steps: usize,
    #[prop(into)] inset_icon: Option<IconContent>,
    controller: Option<State<SliderController>>,
) {
    let args = slider_config_from_params(SliderParams {
        modifier,
        value,
        on_change,
        size,
        active_track_color,
        inactive_track_color,
        thumb_diameter,
        thumb_color,
        disabled,
        accessibility_label,
        accessibility_description,
        show_stop_indicator,
        steps,
        inset_icon,
        controller,
    });
    let controller = args
        .controller
        .unwrap_or_else(|| remember(SliderController::new));
    let mut resolved_args = args;
    resolved_args.controller = Some(controller);
    render_centered_slider(resolved_args);
}

fn render_centered_slider(args: SliderConfig) {
    let controller = args
        .controller
        .expect("render_centered_slider requires controller to be set");
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
    let tap_recognizer = remember(TapRecognizer::default);
    let drag_recognizer = remember(DragRecognizer::default);
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
    );

    layout()
        .modifier(modifier)
        .layout_policy(CenteredSliderLayoutPolicy {
            args: args.clone(),
            clamped_value,
            handle_width,
        })
        .child(move || {
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
            AxisConstraint::exact(segments.left_inactive.1),
            AxisConstraint::exact(layout.base.track_height),
        ),
    )?;
    output.place_child(
        left_inactive_id,
        PxPosition::new(segments.left_inactive.0, track_y),
    );

    input.measure_child(
        active_id,
        &Constraint::new(
            AxisConstraint::exact(segments.active.1),
            AxisConstraint::exact(layout.base.track_height),
        ),
    )?;
    output.place_child(active_id, PxPosition::new(segments.active.0, track_y));

    input.measure_child(
        right_inactive_id,
        &Constraint::new(
            AxisConstraint::exact(segments.right_inactive.1),
            AxisConstraint::exact(layout.base.track_height),
        ),
    )?;
    output.place_child(
        right_inactive_id,
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
            AxisConstraint::exact(stop_size),
            AxisConstraint::exact(stop_size),
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
///   [`RangeSliderConfig`].
/// - `controller` — optional controller; use [`range_slider`] to supply one.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::modifier::ModifierExt as _;
/// use tessera_components::slider::range_slider;
/// use tessera_ui::{Dp, Modifier, remember};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// let range_value = remember(|| (0.2f32, 0.8f32));
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(move || {
/// range_slider()
///     .modifier(Modifier::new().width(Dp(200.0)))
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
    value: (f32, f32),
    on_change: Option<CallbackWith<(f32, f32)>>,
    size: SliderSize,
    active_track_color: Option<Color>,
    inactive_track_color: Option<Color>,
    thumb_diameter: Option<Dp>,
    thumb_color: Option<Color>,
    disabled: bool,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    show_stop_indicator: Option<bool>,
    steps: usize,
    controller: Option<State<RangeSliderController>>,
) {
    let args = range_slider_config_from_params(RangeSliderParams {
        modifier,
        value,
        on_change,
        size,
        active_track_color,
        inactive_track_color,
        thumb_diameter,
        thumb_color,
        disabled,
        accessibility_label,
        accessibility_description,
        show_stop_indicator,
        steps,
        controller,
    });
    let state = args
        .controller
        .unwrap_or_else(|| remember(RangeSliderController::new));
    let mut resolved_args = args;
    resolved_args.controller = Some(state);
    render_range_slider(resolved_args);
}

fn render_range_slider(args: RangeSliderConfig) {
    let state = args
        .controller
        .expect("render_range_slider requires controller to be set");
    let dummy_slider_args = SliderConfig {
        size: args.size,
        show_stop_indicator: args.show_stop_indicator,
        ..SliderConfig::default()
    };
    let initial_width = fallback_component_width(&dummy_slider_args);
    let slider = dummy_slider_args.clone();
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

    let tap_recognizer = remember(TapRecognizer::default);
    let drag_recognizer = remember(DragRecognizer::default);
    let modifier = apply_range_slider_pointer_modifier(
        args.modifier.clone(),
        state,
        args.clone(),
        tap_recognizer,
        drag_recognizer,
        start,
        end,
    );

    layout()
        .modifier(modifier)
        .layout_policy(RangeSliderLayoutPolicy {
            args: args.clone(),
            slider,
            start,
            end,
            start_handle_width,
            end_handle_width,
        })
        .child(move || {
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

            let start_thumb_args = RangeSliderThumbProps {
                thumb_layout: range_layout.base,
                handle_width: start_handle_width,
                colors,
                focus: state.with(|s| s.focus_start),
                accessibility: RangeThumbAccessibility {
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
                        let on_change = args.on_change;
                        move |new_start| on_change.call((new_start, end))
                    }),
                },
            };
            range_slider_thumb()
                .thumb_layout(start_thumb_args.thumb_layout)
                .handle_width(start_thumb_args.handle_width)
                .colors(start_thumb_args.colors)
                .focus(start_thumb_args.focus)
                .accessibility(start_thumb_args.accessibility);

            let end_thumb_args = RangeSliderThumbProps {
                thumb_layout: range_layout.base,
                handle_width: end_handle_width,
                colors,
                focus: state.with(|s| s.focus_end),
                accessibility: RangeThumbAccessibility {
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
                        let on_change = args.on_change;
                        move |new_end| on_change.call((start, new_end))
                    }),
                },
            };
            range_slider_thumb()
                .thumb_layout(end_thumb_args.thumb_layout)
                .handle_width(end_thumb_args.handle_width)
                .colors(end_thumb_args.colors)
                .focus(end_thumb_args.focus)
                .accessibility(end_thumb_args.accessibility);
        });
}
