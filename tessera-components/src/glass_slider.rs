//! A slider component with a glassmorphic visual style.
//!
//! ## Usage
//!
//! Use to select a value from a continuous range.
use derive_setters::Setters;
use tessera_ui::{
    CallbackWith, Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp,
    MeasurementError, Modifier, Px, PxPosition, State,
    accesskit::Role,
    focus_state::Focus,
    layout::{LayoutInput, LayoutOutput, LayoutSpec},
    remember, tessera,
    winit::window::CursorIcon,
};

use crate::{
    fluid_glass::{FluidGlassArgs, GlassBorder, fluid_glass},
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
};

const ACCESSIBILITY_STEP: f32 = 0.05;

/// Controller for the `glass_slider` component.
pub struct GlassSliderController {
    is_dragging: bool,
    focus: Focus,
}

impl GlassSliderController {
    /// Creates a new slider controller.
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            focus: Focus::new(),
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
#[derive(PartialEq, Clone, Setters)]
pub struct GlassSliderArgs {
    /// The current value of the slider, ranging from 0.0 to 1.0.
    pub value: f32,

    /// Layout modifiers applied to the slider track.
    pub modifier: Modifier,

    /// Callback function triggered when the slider's value changes.
    #[setters(skip)]
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
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
    /// Optional external controller for drag and focus state.
    ///
    /// When this is `None`, `glass_slider` creates and owns an internal
    /// controller.
    #[setters(skip)]
    pub controller: Option<State<GlassSliderController>>,
}

impl GlassSliderArgs {
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

    /// Sets an external glass slider controller.
    pub fn controller(mut self, controller: State<GlassSliderController>) -> Self {
        self.controller = Some(controller);
        self
    }
}

impl Default for GlassSliderArgs {
    fn default() -> Self {
        Self {
            value: 0.0,
            modifier: default_slider_modifier(),
            on_change: CallbackWith::new(|_| {}),
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

#[derive(Clone, PartialEq)]
struct GlassSliderProgressFillArgs {
    value: f32,
    tint_color: Color,
    blur_radius: Dp,
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

/// Process cursor events and update the slider state accordingly.
/// Returns the new value (0.0..1.0) if a change should be emitted.
fn process_cursor_events(
    controller: State<GlassSliderController>,
    input: &tessera_ui::InputHandlerInput,
    width_f: f32,
) -> Option<f32> {
    let mut new_value: Option<f32> = None;

    for event in input.cursor_events.iter() {
        match &event.content {
            CursorEventContent::Pressed(_) => {
                controller.with_mut(|c| {
                    c.request_focus();
                    c.set_dragging(true);
                });
                if let Some(v) = cursor_progress(input.cursor_position_rel, width_f) {
                    new_value = Some(v);
                }
            }
            CursorEventContent::Released(_) => {
                controller.with_mut(|c| c.set_dragging(false));
            }
            _ => {}
        }
    }

    if controller.with(|c| c.is_dragging())
        && let Some(v) = cursor_progress(input.cursor_position_rel, width_f)
    {
        new_value = Some(v);
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
///   callback; see [`GlassSliderArgs`].
/// - `controller` — optional controller; use [`glass_slider`] to provide your
///   own.
///
/// ## Examples
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use tessera_components::glass_slider::{GlassSliderArgs, GlassSliderController, glass_slider};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     // In a real app, this would be part of your application's state.
///     let slider_value = Arc::new(Mutex::new(0.5));
///     let slider_controller = remember(GlassSliderController::new);
///
///     let on_change = {
///         let slider_value = slider_value.clone();
///         move |new_value| {
///             *slider_value.lock().unwrap() = new_value;
///         }
///     };
///
///     let args = GlassSliderArgs::default()
///         .value(*slider_value.lock().unwrap())
///         .on_change(on_change)
///         .controller(slider_controller);
///
///     glass_slider(&args);
///
///     // For the doctest, we can simulate the callback.
///     assert_eq!(*slider_value.lock().unwrap(), 0.5);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn glass_slider(args: &GlassSliderArgs) {
    let mut slider_args = args.clone();
    let controller = slider_args
        .controller
        .unwrap_or_else(|| remember(GlassSliderController::new));
    slider_args.controller = Some(controller);
    glass_slider_node(&slider_args);
}

#[tessera]
fn glass_slider_progress_fill_node(args: &GlassSliderProgressFillArgs) {
    let value = args.value;
    let tint_color = args.tint_color;
    let blur_radius = args.blur_radius;
    fluid_glass(&crate::fluid_glass::FluidGlassArgs::with_child(
        FluidGlassArgs::default()
            .tint_color(tint_color)
            .blur_radius(blur_radius)
            .shape(Shape::capsule())
            .refraction_amount(0.0),
        || {},
    ));

    let clamped = value.clamp(0.0, 1.0);
    layout(GlassSliderFillLayout { value: clamped });
}

#[derive(Clone, PartialEq)]
struct GlassSliderFillLayout {
    value: f32,
}

impl LayoutSpec for GlassSliderFillLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let available_width = match input.parent_constraint().width() {
            DimensionValue::Fixed(px) => px,
            DimensionValue::Wrap { max, .. } => max.unwrap_or(Px(0)),
            DimensionValue::Fill { max, .. } => max.expect(
                "Seems that you are trying to fill an infinite width, which is not allowed",
            ),
        };
        let available_height = match input.parent_constraint().height() {
            DimensionValue::Fixed(px) => px,
            DimensionValue::Wrap { max, .. } => max.unwrap_or(Px(0)),
            DimensionValue::Fill { max, .. } => max.expect(
                "Seems that you are trying to fill an infinite height, which is not allowed",
            ),
        };

        let width_px = Px((available_width.to_f32() * self.value).round() as i32);
        let child_id = input
            .children_ids()
            .first()
            .copied()
            .expect("progress fill child should exist");

        let child_constraint = Constraint::new(
            DimensionValue::Fixed(width_px),
            DimensionValue::Fixed(available_height),
        );
        input.measure_child(child_id, &child_constraint)?;
        output.place_child(child_id, PxPosition::new(Px(0), Px(0)));

        Ok(ComputedData {
            width: width_px,
            height: available_height,
        })
    }
}

#[tessera]
fn glass_slider_node(args: &GlassSliderArgs) {
    let args = args.clone();
    let controller = args
        .controller
        .expect("glass_slider_node requires controller to be set");
    let mut modifier = args.modifier.clone();
    let mut semantics = SemanticsArgs::new().role(Role::Slider);
    if let Some(label) = args.accessibility_label.clone() {
        semantics = semantics.label(label);
    }
    if let Some(description) = args.accessibility_description.clone() {
        semantics = semantics.description(description);
    }
    semantics = semantics
        .numeric_range(0.0, 1.0)
        .numeric_value(args.value as f64)
        .numeric_value_step(ACCESSIBILITY_STEP as f64);
    semantics = if args.disabled {
        semantics.disabled(true)
    } else {
        semantics.focusable(true)
    };
    modifier = modifier.semantics(semantics);

    modifier.run(move || {
        let mut inner_args = args.clone();
        inner_args.controller = Some(controller);
        glass_slider_inner_node(&inner_args);
    });
}

#[tessera]
fn glass_slider_inner_node(args: &GlassSliderArgs) {
    let args = args.clone();
    let controller = args
        .controller
        .expect("glass_slider_inner_node requires controller to be set");
    // External track (background) with border - capsule shape
    fluid_glass(&crate::fluid_glass::FluidGlassArgs::with_child(
        FluidGlassArgs::default()
            .modifier(Modifier::new().fill_max_size())
            .tint_color(args.track_tint_color)
            .blur_radius(args.blur_radius)
            .shape(Shape::capsule())
            .border(GlassBorder::new(args.track_border_width.into()))
            .padding(args.track_border_width),
        move || {
            // Internal progress fill - capsule shape using surface
            // Child constraint already excludes padding from the track.
            let fill_args = GlassSliderProgressFillArgs {
                value: args.value,
                tint_color: args.progress_tint_color,
                blur_radius: args.blur_radius,
            };
            glass_slider_progress_fill_node(&fill_args);
        },
    ));

    let on_change = args.on_change.clone();
    let handler_args = args.clone();

    input_handler(move |input| {
        if !handler_args.disabled {
            let is_in_component =
                cursor_within_bounds(input.cursor_position_rel, &input.computed_data);

            if is_in_component {
                input.requests.cursor_icon = CursorIcon::Pointer;
            }

            if is_in_component || controller.with(|c| c.is_dragging()) {
                let width_f = input.computed_data.width.0 as f32;

                if let Some(v) = process_cursor_events(controller, &input, width_f)
                    && (v - handler_args.value).abs() > f32::EPSILON
                {
                    on_change.call(v);
                }
            }
        }
    });
    let mut semantics = SemanticsArgs::new().role(Role::Slider);
    if let Some(label) = args.accessibility_label.clone() {
        semantics = semantics.label(label);
    }
    if let Some(description) = args.accessibility_description.clone() {
        semantics = semantics.description(description);
    }
    semantics = semantics
        .numeric_range(0.0, 1.0)
        .numeric_value(args.value as f64)
        .numeric_value_step(ACCESSIBILITY_STEP as f64);
    semantics = if args.disabled {
        semantics.disabled(true)
    } else {
        semantics.focusable(true)
    };
    let _modifier = Modifier::new().semantics(semantics);

    let track_height = args.track_height.to_px();
    let fallback_width = Dp(200.0).to_px();

    layout(GlassSliderLayout {
        track_height,
        fallback_width,
    });
}

#[derive(Clone, Copy, PartialEq)]
struct GlassSliderLayout {
    track_height: Px,
    fallback_width: Px,
}

impl LayoutSpec for GlassSliderLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let width_dim = input.parent_constraint().width();
        let self_width = match width_dim {
            DimensionValue::Fixed(px) => px,
            DimensionValue::Wrap { max, .. } => max.unwrap_or(self.fallback_width),
            DimensionValue::Fill { max, .. } => max.expect(
                "Seems that you are trying to fill an infinite width, which is not allowed",
            ),
        };
        let self_height = self.track_height;

        let track_id = input.children_ids()[0];
        let track_constraint = Constraint::new(
            DimensionValue::Fixed(self_width),
            DimensionValue::Fixed(self_height),
        );
        input.measure_child(track_id, &track_constraint)?;
        output.place_child(track_id, PxPosition::new(Px(0), Px(0)));

        Ok(ComputedData {
            width: self_width,
            height: self_height,
        })
    }
}
