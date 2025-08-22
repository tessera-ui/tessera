//! A reusable, interactive slider UI component for selecting a value within the range [0.0, 1.0].
//!
//! This module provides a customizable horizontal slider, suitable for use in forms, settings panels,
//! media controls, or any scenario where users need to adjust a continuous value. The slider supports
//! mouse and keyboard interaction, visual feedback for dragging and focus, and allows full control over
//! appearance and behavior via configuration options and callbacks.
//!
//! Typical use cases include volume controls, progress bars, brightness adjustments, and other parameter selection tasks.
//!
//! The slider is fully controlled: you provide the current value and handle updates via a callback.
//! State management (e.g., dragging, focus) is handled externally and passed in, enabling integration with various UI frameworks.
//!
//! See [`SliderArgs`] and [`SliderState`] for configuration and state management details.

use std::sync::Arc;

use derive_builder::Builder;
use parking_lot::Mutex;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, MeasureInput,
    MeasurementError, Px, PxPosition, StateHandlerInput, focus_state::Focus, tessera,
    winit::window::CursorIcon,
};

use crate::{
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

///
/// Stores the interactive state for the [`slider`] component, such as whether the slider is currently being dragged by the user.
/// This struct should be managed via [`Arc<Mutex<SliderState>>`] and passed to the [`slider`] function to enable correct interaction handling.
///
/// # Fields
/// - `is_dragging`: Indicates whether the user is actively dragging the slider thumb.
/// - `focus`: Manages keyboard focus for the slider component.
///
/// Typically, you create and manage this state using [`use_state`] or similar state management utilities.
///
/// [`slider`]: crate::slider
pub struct SliderState {
    /// True if the user is currently dragging the slider.
    pub is_dragging: bool,
    /// The focus handler for the slider.
    pub focus: Focus,
}

impl Default for SliderState {
    fn default() -> Self {
        Self::new()
    }
}

impl SliderState {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            focus: Focus::new(),
        }
    }
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

    /// The width of the slider track.
    #[builder(default = "Dp(200.0)")]
    pub width: Dp,

    /// The height of the slider track.
    #[builder(default = "Dp(12.0)")]
    pub track_height: Dp,

    /// The color of the active part of the track (progress fill).
    #[builder(default = "Color::new(0.2, 0.5, 0.8, 1.0)")]
    pub active_track_color: Color,

    /// The color of the inactive part of the track (background).
    #[builder(default = "Color::new(0.8, 0.8, 0.8, 1.0)")]
    pub inactive_track_color: Color,

    /// Disable interaction.
    #[builder(default = "false")]
    pub disabled: bool,
}

/// Helper: check if a cursor position is within the bounds of a component.
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

fn handle_slider_state(
    input: &mut StateHandlerInput,
    state: &Arc<Mutex<SliderState>>,
    args: &SliderArgs,
) {
    if args.disabled {
        return;
    }

    let mut state = state.lock();
    let is_in_component = cursor_within_component(input.cursor_position_rel, &input.computed_data);

    if is_in_component {
        input.requests.cursor_icon = CursorIcon::Pointer;
    }

    if !is_in_component && !state.is_dragging {
        return;
    }

    let width_f = input.computed_data.width.0 as f32;
    let mut new_value: Option<f32> = None;

    handle_cursor_events(input, &mut state, &mut new_value, width_f);
    update_value_on_drag(input, &state, &mut new_value, width_f);
    notify_on_change(new_value, args);
}

fn handle_cursor_events(
    input: &mut StateHandlerInput,
    state: &mut SliderState,
    new_value: &mut Option<f32>,
    width_f: f32,
) {
    for event in input.cursor_events.iter() {
        match &event.content {
            CursorEventContent::Pressed(_) => {
                state.focus.request_focus();
                state.is_dragging = true;
                if let Some(v) = cursor_progress(input.cursor_position_rel, width_f) {
                    *new_value = Some(v);
                }
            }
            CursorEventContent::Released(_) => {
                state.is_dragging = false;
            }
            _ => {}
        }
    }
}

fn update_value_on_drag(
    input: &StateHandlerInput,
    state: &SliderState,
    new_value: &mut Option<f32>,
    width_f: f32,
) {
    if state.is_dragging {
        if let Some(v) = cursor_progress(input.cursor_position_rel, width_f) {
            *new_value = Some(v);
        }
    }
}

fn notify_on_change(new_value: Option<f32>, args: &SliderArgs) {
    if let Some(v) = new_value {
        if (v - args.value).abs() > f32::EPSILON {
            (args.on_change)(v);
        }
    }
}

fn render_track(args: &SliderArgs) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(args.width.to_px()))
            .height(DimensionValue::Fixed(args.track_height.to_px()))
            .color(args.inactive_track_color)
            .shape({
                let radius = args.track_height.to_px().to_f32() / 2.0;
                Shape::RoundedRectangle {
                    top_left: radius,
                    top_right: radius,
                    bottom_right: radius,
                    bottom_left: radius,
                    g2_k_value: 2.0, // Capsule shape
                }
            })
            .build()
            .unwrap(),
        None,
        move || {
            render_progress_fill(args);
        },
    );
}

fn render_progress_fill(args: &SliderArgs) {
    let progress_width = args.width.to_px().to_f32() * args.value;
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(Px(progress_width as i32)))
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .color(args.active_track_color)
            .shape({
                let radius = args.track_height.to_px().to_f32() / 2.0;
                Shape::RoundedRectangle {
                    top_left: radius,
                    top_right: radius,
                    bottom_right: radius,
                    bottom_left: radius,
                    g2_k_value: 2.0, // Capsule shape
                }
            })
            .build()
            .unwrap(),
        None,
        || {},
    );
}

fn measure_slider(
    input: &MeasureInput,
    args: &SliderArgs,
) -> Result<ComputedData, MeasurementError> {
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
}

/// Renders a slider UI component that allows users to select a value in the range `[0.0, 1.0]`.
///
/// The slider displays a horizontal track with a draggable thumb. The current value is visually represented by the filled portion of the track.
/// The component is fully controlled: you must provide the current value and a callback to handle value changes.
///
/// # Parameters
/// - `args`: Arguments for configuring the slider. See [`SliderArgs`] for all options. The most important are:
///   - `value` (`f32`): The current value of the slider, in the range `[0.0, 1.0]`.
///   - `on_change` (`Arc<dyn Fn(f32) + Send + Sync>`): Callback invoked when the user changes the slider's value.
///   - `width`, `track_height`, `active_track_color`, `inactive_track_color`, `disabled`: Appearance and interaction options.
/// - `state`: Shared state for the slider, used to track interaction (e.g., dragging, focus). Create and manage this using [`use_state`] or similar, and pass it to the slider for correct behavior.
///
/// # State Management
/// The `state` parameter must be an [`Arc<Mutex<SliderState>>`]. You can create and manage it using the `use_state` hook or any other state management approach compatible with your application.
///
/// # Example
/// ```
/// use std::sync::Arc;
/// use parking_lot::Mutex;
/// use tessera_ui::Dp;
/// use tessera_ui_basic_components::slider::{slider, SliderArgs, SliderState, SliderArgsBuilder};
///
/// // In a real application, you would manage the state.
/// let slider_state = Arc::new(Mutex::new(SliderState::new()));
///
/// // Create a slider with a width of 200dp and an initial value of 0.5.
/// slider(
///     SliderArgsBuilder::default()
///         .width(Dp(200.0))
///         .value(0.5)
///         .on_change(Arc::new(|new_value| {
///             // Update your application state here.
///             println!("Slider value: {}", new_value);
///         }))
///         .build()
///         .unwrap(),
///     slider_state,
/// );
/// ```
///
/// This example demonstrates how to create a stateful slider and respond to value changes by updating your own state.
///
/// # See Also
/// - [`SliderArgs`]
/// - [`SliderState`]
/// Helper: check if a cursor position is inside a measured component area.
#[tessera]
pub fn slider(args: impl Into<SliderArgs>, state: Arc<Mutex<SliderState>>) {
    let args: SliderArgs = args.into();

    render_track(&args);

    let cloned_args = args.clone();
    let state_clone = state.clone();
    state_handler(Box::new(move |mut input| {
        handle_slider_state(&mut input, &state_clone, &cloned_args);
    }));

    let cloned_args = args.clone();
    measure(Box::new(move |input| measure_slider(input, &cloned_args)));
}
