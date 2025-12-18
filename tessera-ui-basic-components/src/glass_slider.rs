//! A slider component with a glassmorphic visual style.
//!
//! ## Usage
//!
//! Use to select a value from a continuous range.
use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, Modifier, Px,
    PxPosition, State,
    accesskit::{Action, Role},
    focus_state::Focus,
    remember, tessera,
    winit::window::CursorIcon,
};

use crate::{
    fluid_glass::{FluidGlassArgsBuilder, GlassBorder, fluid_glass},
    modifier::ModifierExt as _,
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
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct GlassSliderArgs {
    /// The current value of the slider, ranging from 0.0 to 1.0.
    #[builder(default = "0.0")]
    pub value: f32,

    /// Layout modifiers applied to the slider track.
    #[builder(default = "default_slider_modifier()")]
    pub modifier: Modifier,

    /// Callback function triggered when the slider's value changes.
    #[builder(default = "Arc::new(|_| {})")]
    pub on_change: Arc<dyn Fn(f32) + Send + Sync>,

    /// The height of the slider track.
    #[builder(default = "Dp(12.0)")]
    pub track_height: Dp,

    /// Glass tint color for the track background.
    #[builder(default = "Color::new(0.3, 0.3, 0.3, 0.15)")]
    pub track_tint_color: Color,

    /// Glass tint color for the progress fill.
    #[builder(default = "Color::new(0.5, 0.7, 1.0, 0.25)")]
    pub progress_tint_color: Color,

    /// Glass blur radius for all components.
    #[builder(default = "Dp(0.0)")]
    pub blur_radius: Dp,

    /// Border width for the track.
    #[builder(default = "Dp(1.0)")]
    pub track_border_width: Dp,

    /// Disable interaction.
    #[builder(default = "false")]
    pub disabled: bool,
    /// Optional accessibility label read by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
}

fn default_slider_modifier() -> Modifier {
    Modifier::new().width(Dp(200.0))
}

/// Helper: check if a cursor position is inside a measured component area.
/// Extracted to reduce duplication and keep the input handler concise.
fn cursor_within_component(cursor_pos: Option<PxPosition>, computed: &ComputedData) -> bool {
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
/// - `controller` — optional controller; use [`glass_slider_with_controller`]
///   to provide your own.
///
/// ## Examples
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use tessera_ui::{remember, tessera};
/// use tessera_ui_basic_components::glass_slider::{
///     GlassSliderArgsBuilder, GlassSliderController, glass_slider_with_controller,
/// };
///
/// #[tessera]
/// fn demo() {
///     // In a real app, this would be part of your application's state.
///     let slider_value = Arc::new(Mutex::new(0.5));
///     let slider_controller = remember(GlassSliderController::new);
///
///     let on_change = {
///         let slider_value = slider_value.clone();
///         Arc::new(move |new_value| {
///             *slider_value.lock().unwrap() = new_value;
///         })
///     };
///
///     let args = GlassSliderArgsBuilder::default()
///         .value(*slider_value.lock().unwrap())
///         .on_change(on_change)
///         .build()
///         .unwrap();
///
///     glass_slider_with_controller(args, slider_controller);
///
///     // For the doctest, we can simulate the callback.
///     assert_eq!(*slider_value.lock().unwrap(), 0.5);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn glass_slider(args: impl Into<GlassSliderArgs>) {
    let args: GlassSliderArgs = args.into();
    let controller = remember(GlassSliderController::new);
    glass_slider_with_controller(args, controller);
}

#[tessera]
fn glass_slider_progress_fill(value: f32, tint_color: Color, blur_radius: Dp) {
    fluid_glass(
        FluidGlassArgsBuilder::default()
            .tint_color(tint_color)
            .blur_radius(blur_radius)
            .shape(Shape::capsule())
            .refraction_amount(0.0)
            .build()
            .expect("builder construction failed"),
        || {},
    );

    let clamped = value.clamp(0.0, 1.0);
    measure(Box::new(move |input| {
        let available_width = match input.parent_constraint.width() {
            DimensionValue::Fixed(px) => px,
            DimensionValue::Wrap { max, .. } => max.unwrap_or(Px(0)),
            DimensionValue::Fill { max, .. } => max.expect(
                "Seems that you are trying to fill an infinite width, which is not allowed",
            ),
        };
        let available_height = match input.parent_constraint.height() {
            DimensionValue::Fixed(px) => px,
            DimensionValue::Wrap { max, .. } => max.unwrap_or(Px(0)),
            DimensionValue::Fill { max, .. } => max.expect(
                "Seems that you are trying to fill an infinite height, which is not allowed",
            ),
        };

        let width_px = Px((available_width.to_f32() * clamped).round() as i32);
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("progress fill child should exist");

        let child_constraint = Constraint::new(
            DimensionValue::Fixed(width_px),
            DimensionValue::Fixed(available_height),
        );
        input.measure_child(child_id, &child_constraint)?;
        input.place_child(child_id, PxPosition::new(Px(0), Px(0)));

        Ok(ComputedData {
            width: width_px,
            height: available_height,
        })
    }));
}

/// # glass_slider_with_controller
///
/// Controlled glass slider variant.
///
/// # Usage
///
/// Use when you need a slider with a glassmorphic style and explicit control
/// over its state.
///
/// # Parameters
///
/// - `args` — configures the slider's value, appearance, and `on_change`
///   callback; see [`GlassSliderArgs`].
/// - `controller` — an explicit [`GlassSliderController`] to manage the
///   slider's state
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui::{remember, tessera};
/// use tessera_ui_basic_components::glass_slider::{
///     GlassSliderArgsBuilder, GlassSliderController, glass_slider_with_controller,
/// };
///
/// #[tessera]
/// fn foo() {
///     let controller = remember(|| GlassSliderController::new());
///     glass_slider_with_controller(
///         GlassSliderArgsBuilder::default()
///             .value(0.3)
///             .on_change(Arc::new(|v| {
///                 println!("Slider value changed to {}", v);
///             }))
///             .build()
///             .unwrap(),
///         controller,
///     );
/// }
/// ```
#[tessera]
pub fn glass_slider_with_controller(
    args: impl Into<GlassSliderArgs>,
    controller: State<GlassSliderController>,
) {
    let args: GlassSliderArgs = args.into();
    let modifier = args.modifier;

    modifier.run(move || glass_slider_inner(args, controller));
}

#[tessera]
fn glass_slider_inner(args: GlassSliderArgs, controller: State<GlassSliderController>) {
    // External track (background) with border - capsule shape
    fluid_glass(
        FluidGlassArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .tint_color(args.track_tint_color)
            .blur_radius(args.blur_radius)
            .shape(Shape::capsule())
            .border(GlassBorder::new(args.track_border_width.into()))
            .padding(args.track_border_width)
            .build()
            .expect("builder construction failed"),
        move || {
            // Internal progress fill - capsule shape using surface
            // Child constraint already excludes padding from the track.
            glass_slider_progress_fill(args.value, args.progress_tint_color, args.blur_radius);
        },
    );

    let on_change = args.on_change.clone();
    let args_for_handler = args.clone();

    input_handler(Box::new(move |mut input| {
        if !args_for_handler.disabled {
            let is_in_component =
                cursor_within_component(input.cursor_position_rel, &input.computed_data);

            if is_in_component {
                input.requests.cursor_icon = CursorIcon::Pointer;
            }

            if is_in_component || controller.with(|c| c.is_dragging()) {
                let width_f = input.computed_data.width.0 as f32;

                if let Some(v) = process_cursor_events(controller, &input, width_f)
                    && (v - args_for_handler.value).abs() > f32::EPSILON
                {
                    on_change(v);
                }
            }
        }

        apply_glass_slider_accessibility(
            &mut input,
            &args_for_handler,
            args_for_handler.value,
            &args_for_handler.on_change,
        );
    }));

    let track_height = args.track_height.to_px();
    let fallback_width = Dp(200.0).to_px();

    measure(Box::new(move |input| {
        let width_dim = input.parent_constraint.width();
        let self_width = match width_dim {
            DimensionValue::Fixed(px) => px,
            DimensionValue::Wrap { max, .. } => max.unwrap_or(fallback_width),
            DimensionValue::Fill { max, .. } => max.expect(
                "Seems that you are trying to fill an infinite width, which is not allowed",
            ),
        };
        let self_height = track_height;

        let track_id = input.children_ids[0];

        // Measure track
        let track_constraint = Constraint::new(
            DimensionValue::Fixed(self_width),
            DimensionValue::Fixed(self_height),
        );
        input.measure_child(track_id, &track_constraint)?;
        input.place_child(track_id, PxPosition::new(Px(0), Px(0)));

        Ok(ComputedData {
            width: self_width,
            height: self_height,
        })
    }));
}

fn apply_glass_slider_accessibility(
    input: &mut tessera_ui::InputHandlerInput<'_>,
    args: &GlassSliderArgs,
    current_value: f32,
    on_change: &Arc<dyn Fn(f32) + Send + Sync>,
) {
    let mut builder = input.accessibility().role(Role::Slider);

    if let Some(label) = args.accessibility_label.as_ref() {
        builder = builder.label(label.clone());
    }
    if let Some(description) = args.accessibility_description.as_ref() {
        builder = builder.description(description.clone());
    }

    builder = builder
        .numeric_value(current_value as f64)
        .numeric_range(0.0, 1.0);

    if args.disabled {
        builder = builder.disabled();
    } else {
        builder = builder
            .action(Action::Increment)
            .action(Action::Decrement)
            .focusable();
    }

    builder.commit();

    if args.disabled {
        return;
    }

    let on_change = on_change.clone();
    input.set_accessibility_action_handler(move |action| {
        let new_value = match action {
            Action::Increment => Some((current_value + ACCESSIBILITY_STEP).clamp(0.0, 1.0)),
            Action::Decrement => Some((current_value - ACCESSIBILITY_STEP).clamp(0.0, 1.0)),
            _ => None,
        };

        if let Some(new_value) = new_value
            && (new_value - current_value).abs() > f32::EPSILON
        {
            on_change(new_value);
        }
    });
}
