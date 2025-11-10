//! Provides a glassmorphism-style slider component for selecting a value in modern UI applications.
//!
//! The `glass_slider` module implements a customizable, frosted glass effect slider with support for
//! blurred backgrounds, tint colors, borders, and interactive state management. It enables users to
//! select a continuous value between 0.0 and 1.0 by dragging a thumb along a track, and is suitable
//! for dashboards, settings panels, or any interface requiring visually appealing value selection.
//!
//! Typical usage involves integrating the slider into a component tree, passing state via `Arc<RwLock<GlassSliderState>>`,
//! and customizing appearance through `GlassSliderArgs`. The component is designed to fit seamlessly into
//! glassmorphism-themed user interfaces.
//!
//! See the module-level documentation and examples for details.

use std::sync::Arc;

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, Px, PxPosition,
    accesskit::{Action, Role},
    focus_state::Focus,
    tessera,
    winit::window::CursorIcon,
};

use crate::{
    fluid_glass::{FluidGlassArgsBuilder, GlassBorder, fluid_glass},
    shape_def::Shape,
};

const ACCESSIBILITY_STEP: f32 = 0.05;

/// State for the `glass_slider` component.
pub struct GlassSliderState {
    /// True if the user is currently dragging the slider.
    pub is_dragging: bool,
    /// The focus handler for the slider.
    pub focus: Focus,
}

impl Default for GlassSliderState {
    fn default() -> Self {
        Self::new()
    }
}

impl GlassSliderState {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            focus: Focus::new(),
        }
    }
}

/// Arguments for the `glass_slider` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct GlassSliderArgs {
    /// The current value of the slider, ranging from 0.0 to 1.0.
    #[builder(default = "0.0")]
    pub value: f32,

    /// Callback function triggered when the slider's value changes.
    #[builder(default = "Arc::new(|_| {})")]
    pub on_change: Arc<dyn Fn(f32) + Send + Sync>,

    /// The width of the slider track.
    #[builder(default = "Dp(200.0)")]
    pub width: Dp,

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
    #[builder(default = "0.0")]
    pub blur_radius: f32,

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

/// Helper: compute progress fill width in Px, clamped to >= 0.
fn compute_progress_width(total_width: Px, value: f32, border_padding_px: f32) -> Px {
    let total_f = total_width.0 as f32;
    let mut w = total_f * value - border_padding_px;
    if w < 0.0 {
        w = 0.0;
    }
    Px(w as i32)
}

/// Process cursor events and update the slider state accordingly.
/// Returns the new value (0.0..1.0) if a change should be emitted.
fn process_cursor_events(
    state: &mut GlassSliderState,
    input: &tessera_ui::InputHandlerInput,
    width_f: f32,
) -> Option<f32> {
    let mut new_value: Option<f32> = None;

    for event in input.cursor_events.iter() {
        match &event.content {
            CursorEventContent::Pressed(_) => {
                state.focus.request_focus();
                state.is_dragging = true;
                if let Some(v) = cursor_progress(input.cursor_position_rel, width_f) {
                    new_value = Some(v);
                }
            }
            CursorEventContent::Released(_) => {
                state.is_dragging = false;
            }
            _ => {}
        }
    }

    if state.is_dragging
        && let Some(v) = cursor_progress(input.cursor_position_rel, width_f)
    {
        new_value = Some(v);
    }

    new_value
}

/// Creates a slider component with a frosted glass effect.
///
/// The `glass_slider` allows users to select a value from a continuous range (0.0 to 1.0)
/// by dragging a handle along a track. It features a modern, semi-transparent
/// "glassmorphism" aesthetic, with a blurred background and subtle highlights.
///
/// # Arguments
///
/// * `args` - An instance of `GlassSliderArgs` or `GlassSliderArgsBuilder` to configure the slider's appearance and behavior.
///   - `value`: The current value of the slider, must be between 0.0 and 1.0.
///   - `on_change`: A callback function that is triggered when the slider's value changes.
///     It receives the new value as an `f32`.
/// * `state` - An `Arc<RwLock<GlassSliderState>>` to manage the component's interactive state,
///   such as dragging and focus.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use parking_lot::RwLock;
/// use tessera_ui_basic_components::glass_slider::{glass_slider, GlassSliderArgsBuilder, GlassSliderState};
///
/// // In your application state
/// let slider_value = Arc::new(RwLock::new(0.5));
/// let slider_state = Arc::new(RwLock::new(GlassSliderState::new()));
///
/// // In your component function
/// let on_change_callback = {
///     let slider_value = slider_value.clone();
///     Arc::new(move |new_value| {
///         *slider_value.write() = new_value;
///     })
/// };
///
/// glass_slider(
///     GlassSliderArgsBuilder::default()
///         .value(*slider_value.read())
///         .on_change(on_change_callback)
///         .build()
///         .unwrap(),
///     slider_state.clone(),
/// );
/// ```
#[tessera]
pub fn glass_slider(args: impl Into<GlassSliderArgs>, state: Arc<RwLock<GlassSliderState>>) {
    let args: GlassSliderArgs = args.into();
    let border_padding_px = args.track_border_width.to_px().to_f32() * 2.0;

    // External track (background) with border - capsule shape
    fluid_glass(
        FluidGlassArgsBuilder::default()
            .width(DimensionValue::Fixed(args.width.to_px()))
            .height(DimensionValue::Fixed(args.track_height.to_px()))
            .tint_color(args.track_tint_color)
            .blur_radius(args.blur_radius)
            .shape({
                let track_radius_dp = Dp(args.track_height.0 / 2.0);
                Shape::RoundedRectangle {
                    top_left: track_radius_dp,
                    top_right: track_radius_dp,
                    bottom_right: track_radius_dp,
                    bottom_left: track_radius_dp,
                    g2_k_value: 2.0, // Capsule shape
                }
            })
            .border(GlassBorder::new(args.track_border_width.into()))
            .padding(args.track_border_width)
            .build()
            .unwrap(),
        None,
        move || {
            // Internal progress fill - capsule shape using surface
            let progress_width_px =
                compute_progress_width(args.width.to_px(), args.value, border_padding_px);
            let effective_height = args.track_height.to_px().to_f32() - border_padding_px;
            fluid_glass(
                FluidGlassArgsBuilder::default()
                    .width(DimensionValue::Fixed(progress_width_px))
                    .height(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .tint_color(args.progress_tint_color)
                    .shape({
                        let effective_height_dp = Dp::from_pixels_f32(effective_height);
                        let radius_dp = Dp(effective_height_dp.0 / 2.0);
                        Shape::RoundedRectangle {
                            top_left: radius_dp,
                            top_right: radius_dp,
                            bottom_right: radius_dp,
                            bottom_left: radius_dp,
                            g2_k_value: 2.0, // Capsule shape
                        }
                    })
                    .refraction_amount(0.0)
                    .build()
                    .unwrap(),
                None,
                || {},
            );
        },
    );

    let on_change = args.on_change.clone();
    let args_for_handler = args.clone();
    let state_for_handler = state.clone();
    input_handler(Box::new(move |mut input| {
        if !args_for_handler.disabled {
            let is_in_component =
                cursor_within_component(input.cursor_position_rel, &input.computed_data);

            if is_in_component {
                input.requests.cursor_icon = CursorIcon::Pointer;
            }

            if is_in_component || state_for_handler.read().is_dragging {
                let width_f = input.computed_data.width.0 as f32;

                if let Some(v) =
                    process_cursor_events(&mut state_for_handler.write(), &input, width_f)
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

    measure(Box::new(move |input| {
        let self_width = args.width.to_px();
        let self_height = args.track_height.to_px();

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

    let value_for_handler = current_value;
    let on_change = on_change.clone();
    input.set_accessibility_action_handler(move |action| {
        let new_value = match action {
            Action::Increment => Some((value_for_handler + ACCESSIBILITY_STEP).clamp(0.0, 1.0)),
            Action::Decrement => Some((value_for_handler - ACCESSIBILITY_STEP).clamp(0.0, 1.0)),
            _ => None,
        };

        if let Some(new_value) = new_value
            && (new_value - value_for_handler).abs() > f32::EPSILON
        {
            on_change(new_value);
        }
    });
}
