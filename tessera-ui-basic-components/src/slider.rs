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
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, Px, PxPosition,
    focus_state::Focus, winit::window::CursorIcon,
};
use tessera_ui_macros::tessera;

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

#[tessera]
///
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
pub fn slider(args: impl Into<SliderArgs>, state: Arc<Mutex<SliderState>>) {
    let args: SliderArgs = args.into();

    // Background track (inactive part) - capsule shape
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(args.width.to_px()))
            .height(DimensionValue::Fixed(args.track_height.to_px()))
            .color(args.inactive_track_color)
            .shape(Shape::RoundedRectangle {
                corner_radius: args.track_height.to_px().to_f32() / 2.0,
                g2_k_value: 2.0, // Capsule shape
            })
            .build()
            .unwrap(),
        None,
        move || {
            // Progress fill (active part) - capsule shape
            let progress_width = args.width.to_px().to_f32() * args.value;
            surface(
                SurfaceArgsBuilder::default()
                    .width(DimensionValue::Fixed(Px(progress_width as i32)))
                    .height(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .color(args.active_track_color)
                    .shape(Shape::RoundedRectangle {
                        corner_radius: args.track_height.to_px().to_f32() / 2.0,
                        g2_k_value: 2.0, // Capsule shape
                    })
                    .build()
                    .unwrap(),
                None,
                || {},
            );
        },
    );

    let on_change = args.on_change.clone();
    let state_handler_state = state.clone();
    let disabled = args.disabled;

    state_handler(Box::new(move |input| {
        if disabled {
            return;
        }
        let mut state = state_handler_state.lock();

        let is_in_component = input.cursor_position_rel.is_some_and(|cursor_pos| {
            cursor_pos.x.0 >= 0
                && cursor_pos.x.0 < input.computed_data.width.0
                && cursor_pos.y.0 >= 0
                && cursor_pos.y.0 < input.computed_data.height.0
        });

        // Set cursor to pointer when hovering over the slider
        if is_in_component {
            input.requests.cursor_icon = CursorIcon::Pointer;
        }

        if !is_in_component && !state.is_dragging {
            return;
        }

        let mut new_value = None;

        for event in input.cursor_events.iter() {
            match &event.content {
                CursorEventContent::Pressed(_) => {
                    state.focus.request_focus();
                    state.is_dragging = true;

                    if let Some(pos) = input.cursor_position_rel {
                        let v =
                            (pos.x.0 as f32 / input.computed_data.width.0 as f32).clamp(0.0, 1.0);
                        new_value = Some(v);
                    }
                }
                CursorEventContent::Released(_) => {
                    state.is_dragging = false;
                }
                _ => {}
            }
        }

        if state.is_dragging {
            if let Some(pos) = input.cursor_position_rel {
                let v = (pos.x.0 as f32 / input.computed_data.width.0 as f32).clamp(0.0, 1.0);
                new_value = Some(v);
            }
        }

        if let Some(v) = new_value {
            if (v - args.value).abs() > f32::EPSILON {
                on_change(v);
            }
        }
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
