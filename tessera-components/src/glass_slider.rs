//! A slider component with a glassmorphic visual style.
//!
//! ## Usage
//!
//! Use to select a value from a continuous range.
use tessera_foundation::gesture::{DragRecognizer, TapRecognizer};
use tessera_ui::{
    CallbackWith, Color, ComputedData, Constraint, Dp, FocusProperties, FocusRequester,
    LayoutResult, MeasurementError, Modifier, PointerInput, PointerInputModifierNode, Px,
    PxPosition, State,
    accesskit::Role,
    layout::{LayoutPolicy, MeasureScope, layout},
    modifier::{CursorModifierExt as _, FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera,
    winit::window::CursorIcon,
};

use crate::{
    fluid_glass::{GlassBorder, fluid_glass},
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
};

const ACCESSIBILITY_STEP: f32 = 0.05;

struct GlassSliderPointerModifierNode {
    controller: State<GlassSliderController>,
    args: GlassSliderConfig,
    tap_recognizer: State<TapRecognizer>,
    drag_recognizer: State<DragRecognizer>,
}

impl PointerInputModifierNode for GlassSliderPointerModifierNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        if self.args.disabled {
            return;
        }

        let is_in_component = cursor_within_bounds(input.cursor_position_rel, &input.computed_data);

        if is_in_component || self.controller.with(|controller| controller.is_dragging()) {
            let width_f = input.computed_data.width.0 as f32;

            if let Some(value) = process_pointer_gestures(
                self.controller,
                self.tap_recognizer,
                self.drag_recognizer,
                &mut input,
                width_f,
            ) && (value - self.args.value).abs() > f32::EPSILON
            {
                self.args.on_change.call(value);
            }
        }
    }
}

fn apply_glass_slider_pointer_modifier(
    base: Modifier,
    controller: State<GlassSliderController>,
    args: GlassSliderConfig,
    tap_recognizer: State<TapRecognizer>,
    drag_recognizer: State<DragRecognizer>,
) -> Modifier {
    let modifier = if !args.disabled {
        base.hover_cursor_icon(CursorIcon::Pointer)
    } else {
        base
    };
    modifier.push_pointer_input(GlassSliderPointerModifierNode {
        controller,
        args,
        tap_recognizer,
        drag_recognizer,
    })
}

/// Controller for the `glass_slider` component.
pub struct GlassSliderController {
    is_dragging: bool,
    focus: FocusRequester,
}

impl GlassSliderController {
    /// Creates a new slider controller.
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            focus: FocusRequester::new(),
        }
    }

    /// Returns whether the slider thumb is currently being dragged.
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Sets the dragging state manually. This allows custom gesture handling.
    pub fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
    }

    /// Requests focus for this slider instance.
    pub fn request_focus(&mut self) {
        self.focus.request_focus();
    }

    /// Clears focus from this slider.
    pub fn clear_focus(&mut self) {
        self.focus.unfocus();
    }

    /// Returns `true` if the slider currently has focus.
    pub fn is_focused(&self) -> bool {
        self.focus.is_focused()
    }
}

impl Default for GlassSliderController {
    fn default() -> Self {
        Self::new()
    }
}

/// Arguments for the `glass_slider` component.
#[derive(Clone, PartialEq)]
struct GlassSliderConfig {
    /// The current value of the slider, ranging from 0.0 to 1.0.
    pub value: f32,

    /// Layout modifiers applied to the slider track.
    pub modifier: Modifier,

    /// Callback function triggered when the slider's value changes.
    pub on_change: CallbackWith<f32>,

    /// The height of the slider track.
    pub track_height: Dp,

    /// Glass tint color for the track background.
    pub track_tint_color: Color,

    /// Glass tint color for the progress fill.
    pub progress_tint_color: Color,

    /// Glass blur radius for all components.
    pub blur_radius: Dp,

    /// Border width for the track.
    pub track_border_width: Dp,

    /// Disable interaction.
    pub disabled: bool,
    /// Optional accessibility label read by assistive technologies.
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    pub accessibility_description: Option<String>,
    /// Optional external controller for drag and focus state.
    ///
    /// When this is `None`, `glass_slider` creates and owns an internal
    /// controller.
    pub controller: Option<State<GlassSliderController>>,
}

impl Default for GlassSliderConfig {
    fn default() -> Self {
        Self {
            value: 0.0,
            modifier: default_slider_modifier(),
            on_change: CallbackWith::default_value(),
            track_height: Dp(12.0),
            track_tint_color: Color::new(0.3, 0.3, 0.3, 0.15),
            progress_tint_color: Color::new(0.5, 0.7, 1.0, 0.25),
            blur_radius: Dp(0.0),
            track_border_width: Dp(1.0),
            disabled: false,
            accessibility_label: None,
            accessibility_description: None,
            controller: None,
        }
    }
}

struct GlassSliderParams {
    value: f32,
    modifier: Option<Modifier>,
    on_change: Option<CallbackWith<f32>>,
    track_height: Option<Dp>,
    track_tint_color: Option<Color>,
    progress_tint_color: Option<Color>,
    blur_radius: Option<Dp>,
    track_border_width: Option<Dp>,
    disabled: bool,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    controller: Option<State<GlassSliderController>>,
}

fn glass_slider_config_from_params(params: GlassSliderParams) -> GlassSliderConfig {
    let defaults = GlassSliderConfig::default();
    GlassSliderConfig {
        value: params.value,
        modifier: params.modifier.unwrap_or(defaults.modifier),
        on_change: params.on_change.unwrap_or_else(CallbackWith::default_value),
        track_height: params.track_height.unwrap_or(defaults.track_height),
        track_tint_color: params.track_tint_color.unwrap_or(defaults.track_tint_color),
        progress_tint_color: params
            .progress_tint_color
            .unwrap_or(defaults.progress_tint_color),
        blur_radius: params.blur_radius.unwrap_or(defaults.blur_radius),
        track_border_width: params
            .track_border_width
            .unwrap_or(defaults.track_border_width),
        disabled: params.disabled,
        accessibility_label: params.accessibility_label,
        accessibility_description: params.accessibility_description,
        controller: params.controller,
    }
}

fn default_slider_modifier() -> Modifier {
    Modifier::new().width(Dp(200.0))
}

/// Helper: check if a cursor position is inside a measured component area.
/// Extracted to reduce duplication and keep the input handler concise.
fn cursor_within_bounds(cursor_pos: Option<PxPosition>, computed: &ComputedData) -> bool {
    if let Some(pos) = cursor_pos {
        let within_x = pos.x.0 >= 0 && pos.x.0 < computed.width.0;
        let within_y = pos.y.0 >= 0 && pos.y.0 < computed.height.0;
        within_x && within_y
    } else {
        false
    }
}

/// Helper: compute normalized progress (0.0..1.0) from cursor X and width.
/// Returns None when cursor is not available.
fn cursor_progress(cursor_pos: Option<PxPosition>, width_f: f32) -> Option<f32> {
    cursor_pos.map(|pos| (pos.x.0 as f32 / width_f).clamp(0.0, 1.0))
}

/// Process pointer gestures and update the slider state accordingly.
/// Returns the new value (0.0..1.0) if a change should be emitted.
fn process_pointer_gestures(
    controller: State<GlassSliderController>,
    tap_recognizer: State<TapRecognizer>,
    drag_recognizer: State<DragRecognizer>,
    input: &mut tessera_ui::PointerInput,
    width_f: f32,
) -> Option<f32> {
    let is_in_component = cursor_within_bounds(input.cursor_position_rel, &input.computed_data);
    let tap_result = tap_recognizer.with_mut(|recognizer| {
        recognizer.update(
            input.pass,
            input.pointer_changes.as_mut_slice(),
            input.cursor_position_rel,
            is_in_component,
        )
    });
    let drag_result = drag_recognizer.with_mut(|recognizer| {
        recognizer.update(
            input.pass,
            input.pointer_changes.as_mut_slice(),
            input.cursor_position_rel,
            is_in_component,
        )
    });

    let mut new_value: Option<f32> = None;

    if tap_result.pressed {
        controller.with_mut(|c| {
            c.request_focus();
        });
        if let Some(v) = cursor_progress(input.cursor_position_rel, width_f) {
            new_value = Some(v);
        }
    }

    if drag_result.started {
        controller.with_mut(|c| c.set_dragging(true));
    }

    if (drag_result.updated || controller.with(|c| c.is_dragging()))
        && let Some(v) = cursor_progress(input.cursor_position_rel, width_f)
    {
        new_value = Some(v);
    }

    if tap_result.released || drag_result.ended {
        controller.with_mut(|c| c.set_dragging(false));
    }

    new_value
}

/// # glass_slider
///
/// Renders an interactive slider with a customizable glass effect.
///
/// ## Usage
///
/// Allow users to select a value from a continuous range (0.0 to 1.0) by
/// dragging a thumb.
///
/// ## Parameters
///
/// - `args` — configures the slider's value, appearance, and `on_change`
///   callback through the component's builder parameters.
/// - `controller` — optional controller; use [`glass_slider`] to provide your
///   own.
///
/// ## Examples
///
/// ```
/// use tessera_components::glass_slider::{GlassSliderController, glass_slider};
/// use tessera_ui::{LayoutResult, remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let slider_value = remember(|| 0.5f32);
///     let slider_controller = remember(GlassSliderController::new);
///
///     glass_slider()
///         .value(slider_value.get())
///         .on_change(move |new_value| {
///             slider_value.set(new_value);
///         })
///         .controller(slider_controller);
///
///     assert_eq!(slider_value.get(), 0.5);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn glass_slider(
    value: Option<f32>,
    modifier: Option<Modifier>,
    on_change: Option<CallbackWith<f32>>,
    track_height: Option<Dp>,
    track_tint_color: Option<Color>,
    progress_tint_color: Option<Color>,
    blur_radius: Option<Dp>,
    track_border_width: Option<Dp>,
    disabled: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    controller: Option<State<GlassSliderController>>,
) {
    let defaults = GlassSliderConfig::default();
    let value = value.unwrap_or(defaults.value);
    let disabled = disabled.unwrap_or(defaults.disabled);
    let mut slider_args = glass_slider_config_from_params(GlassSliderParams {
        value,
        modifier,
        on_change,
        track_height,
        track_tint_color,
        progress_tint_color,
        blur_radius,
        track_border_width,
        disabled,
        accessibility_label,
        accessibility_description,
        controller,
    });
    let controller = slider_args
        .controller
        .unwrap_or_else(|| remember(GlassSliderController::new));
    slider_args.controller = Some(controller);
    render_glass_slider(slider_args);
}

#[tessera]
fn glass_slider_progress_fill(
    value: Option<f32>,
    tint_color: Option<Color>,
    blur_radius: Option<Dp>,
) {
    let value = value.unwrap_or(0.0);
    let tint_color = tint_color.unwrap_or(Color::TRANSPARENT);
    let blur_radius = blur_radius.unwrap_or(Dp(0.0));
    fluid_glass()
        .tint_color(tint_color)
        .blur_radius(blur_radius)
        .shape(Shape::CAPSULE)
        .with_child(|| {});

    let clamped = value.clamp(0.0, 1.0);
    layout().layout_policy(GlassSliderFillLayout { value: clamped });
}

#[derive(Clone, PartialEq)]
struct GlassSliderFillLayout {
    value: f32,
}

impl LayoutPolicy for GlassSliderFillLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let available_width = input
            .parent_constraint()
            .width()
            .resolve_max()
            .unwrap_or(Px(0));
        let available_height = input
            .parent_constraint()
            .height()
            .resolve_max()
            .unwrap_or(Px(0));

        let width_px = Px((available_width.to_f32() * self.value).round() as i32);
        let child = input
            .children()
            .first()
            .copied()
            .expect("progress fill child should exist");

        let child_constraint = Constraint::exact(width_px, available_height);
        child.measure(&child_constraint)?;
        result.place_child(child, PxPosition::new(Px(0), Px(0)));

        Ok(result.with_size(ComputedData {
            width: width_px,
            height: available_height,
        }))
    }
}

fn render_glass_slider(args: GlassSliderConfig) {
    let controller = args
        .controller
        .expect("render_glass_slider requires controller to be set");
    let mut modifier = args.modifier.clone();
    let semantics = SemanticsArgs {
        role: Some(Role::Slider),
        label: args.accessibility_label.clone(),
        description: args.accessibility_description.clone(),
        numeric_value: Some(args.value as f64),
        numeric_range: Some((0.0, 1.0)),
        focusable: !args.disabled,
        disabled: args.disabled,
        numeric_value_step: Some(ACCESSIBILITY_STEP as f64),
        ..Default::default()
    };
    modifier = modifier.semantics(semantics);
    let tap_recognizer = remember(TapRecognizer::default);
    let drag_recognizer = remember(DragRecognizer::default);
    let modifier = apply_glass_slider_pointer_modifier(
        modifier.then(
            Modifier::new()
                .focus_requester(controller.with(|c| c.focus))
                .focusable()
                .focus_properties(
                    FocusProperties::new()
                        .can_focus(!args.disabled)
                        .can_request_focus(!args.disabled),
                ),
        ),
        controller,
        args.clone(),
        tap_recognizer,
        drag_recognizer,
    );

    layout()
        .modifier(modifier)
        .layout_policy(GlassSliderLayout {
            track_height: args.track_height.to_px(),
            fallback_width: Dp(200.0).to_px(),
        })
        .child(move || {
            fluid_glass()
                .modifier(Modifier::new().fill_max_size())
                .tint_color(args.track_tint_color)
                .blur_radius(args.blur_radius)
                .shape(Shape::CAPSULE)
                .border(GlassBorder::new(args.track_border_width.into()))
                .padding(args.track_border_width)
                .with_child(move || {
                    glass_slider_progress_fill()
                        .value(args.value)
                        .tint_color(args.progress_tint_color)
                        .blur_radius(args.blur_radius);
                });
        });
}

#[derive(Clone, Copy, PartialEq)]
struct GlassSliderLayout {
    track_height: Px,
    fallback_width: Px,
}

impl LayoutPolicy for GlassSliderLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let self_width = input.parent_constraint().width().clamp(self.fallback_width);
        let self_height = self.track_height;

        let track = input.children()[0];
        let track_constraint = Constraint::exact(self_width, self_height);
        track.measure(&track_constraint)?;
        result.place_child(track, PxPosition::new(Px(0), Px(0)));

        Ok(result.with_size(ComputedData {
            width: self_width,
            height: self_height,
        }))
    }
}
