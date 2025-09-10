//! A customizable, animated checkbox UI component for Tessera UI.
//!
//! This module provides a standard checkbox widget with support for animated checkmark transitions,
//! external or internal state management, and flexible styling options. The checkbox can be used
//! wherever a boolean selection is required, such as forms, settings panels, or interactive lists.
//!
//! Features include:
//! - Smooth checkmark animation on toggle
//! - Optional external state for advanced control and animation
//! - Customizable size, colors, shape, and hover effects
//! - Callback for state changes to integrate with application logic
//!
//! Typical usage involves passing [`CheckboxArgs`] to the [`checkbox`] function, with optional
//! state sharing for animation or controlled components.
//!
//! Suitable for both simple and complex UI scenarios requiring a responsive, visually appealing checkbox.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp, tessera};

use crate::{
    RippleState,
    alignment::Alignment,
    boxed::{BoxedArgsBuilder, boxed},
    checkmark::{CheckmarkArgsBuilder, checkmark},
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

#[derive(Clone, Default)]
pub struct CheckboxState {
    pub ripple: Arc<RippleState>,
    pub checkmark: Arc<RwLock<CheckmarkState>>,
}

/// Arguments for the `checkbox` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct CheckboxArgs {
    #[builder(default = "Arc::new(|_| {})")]
    pub on_toggle: Arc<dyn Fn(bool) + Send + Sync>,

    #[builder(default = "Dp(24.0)")]
    pub size: Dp,

    #[builder(default = "Color::new(0.8, 0.8, 0.8, 1.0)")]
    pub color: Color,

    #[builder(default = "Color::new(0.6, 0.7, 0.9, 1.0)")]
    pub checked_color: Color,

    #[builder(default = "Color::from_rgb_u8(119, 72, 146)")]
    pub checkmark_color: Color,

    #[builder(default = "5.0")]
    pub checkmark_stroke_width: f32,

    #[builder(default = "1.0")]
    pub checkmark_animation_progress: f32,

    #[builder(
        default = "Shape::RoundedRectangle{ top_left: 4.0, top_right: 4.0, bottom_right: 4.0, bottom_left: 4.0, g2_k_value: 3.0 }"
    )]
    pub shape: Shape,

    #[builder(default)]
    pub hover_color: Option<Color>,
}

impl Default for CheckboxArgs {
    fn default() -> Self {
        CheckboxArgsBuilder::default().build().unwrap()
    }
}

// Animation duration for the checkmark stroke (milliseconds)
const CHECKMARK_ANIMATION_DURATION: Duration = Duration::from_millis(200);

/// State for checkmark animation (similar to `SwitchState`)
pub struct CheckmarkState {
    pub checked: bool,
    progress: f32,
    last_toggle_time: Option<Instant>,
}

impl Default for CheckmarkState {
    fn default() -> Self {
        Self::new(false)
    }
}

impl CheckmarkState {
    pub fn new(initial_state: bool) -> Self {
        Self {
            checked: initial_state,
            progress: if initial_state { 1.0 } else { 0.0 },
            last_toggle_time: None,
        }
    }

    /// Toggle checked state and start animation
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
        self.last_toggle_time = Some(Instant::now());
    }

    /// Update progress based on elapsed time
    pub fn update_progress(&mut self) {
        if let Some(start) = self.last_toggle_time {
            let elapsed = start.elapsed();
            let fraction =
                (elapsed.as_secs_f32() / CHECKMARK_ANIMATION_DURATION.as_secs_f32()).min(1.0);
            self.progress = if self.checked {
                fraction
            } else {
                1.0 - fraction
            };
            if fraction >= 1.0 {
                self.last_toggle_time = None; // Animation ends
            }
        }
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }
}

/// Renders a checkbox component.
///
/// The checkbox is a standard UI element that allows users to select or deselect an option.
/// It visually represents its state, typically as a square box that is either empty or contains a checkmark.
/// The component handles its own animation and state transitions.
///
/// # Arguments
///
/// The component is configured by passing `CheckboxArgs` and a `CheckboxState`.
///
/// * `on_toggle`: A callback function `Arc<dyn Fn(bool) + Send + Sync>` that is invoked when the user
///   clicks the checkbox. It receives the new `checked` state as an argument, allowing the
///   application state to be updated.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use parking_lot::RwLock;
/// use tessera_ui_basic_components::checkbox::{checkbox, CheckboxArgs, CheckboxState, CheckmarkState};
///
/// // Create a checkbox that is initially unchecked.
/// let unchecked_state = Arc::new(CheckboxState::default());
/// checkbox(
///     CheckboxArgs {
///         on_toggle: Arc::new(|new_state| {
///             // In a real app, you would update your state here.
///             println!("Checkbox toggled to: {}", new_state);
///         }),
///         ..Default::default()
///     },
///     unchecked_state,
/// );
///
/// // Create a checkbox that is initially checked.
/// let checked_state = Arc::new(CheckboxState {
///     checkmark: Arc::new(RwLock::new(CheckmarkState::new(true))),
///     ..Default::default()
/// });
/// checkbox(CheckboxArgs::default(), checked_state);
/// ```
#[tessera]
pub fn checkbox(args: impl Into<CheckboxArgs>, state: Arc<CheckboxState>) {
    let args: CheckboxArgs = args.into();

    // If a state is provided, set up an updater to advance the animation each frame
    let checkmark_state = state.checkmark.clone();
    state_handler(Box::new(move |_input| {
        checkmark_state.write().update_progress();
    }));

    // Click handler: toggle animation state if present, otherwise simply forward toggle callback
    let on_click = {
        let state = state.clone();
        let on_toggle = args.on_toggle.clone();
        Arc::new(move || {
            state.checkmark.write().toggle();
            on_toggle(state.checkmark.read().checked);
        })
    };

    let ripple_state = state.ripple.clone();

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(args.size.to_px()))
            .height(DimensionValue::Fixed(args.size.to_px()))
            .style(
                if state.checkmark.read().checked {
                    args.checked_color
                } else {
                    args.color
                }
                .into(),
            )
            .hover_style(args.hover_color.map(|c| c.into()))
            .shape(args.shape)
            .on_click(on_click)
            .build()
            .unwrap(),
        Some(ripple_state),
        {
            let state_for_child = state.clone();
            move || {
                let progress = state_for_child.checkmark.read().progress();
                if progress > 0.0 {
                    surface(
                        SurfaceArgsBuilder::default()
                            .padding(Dp(2.0))
                            .style(Color::TRANSPARENT.into())
                            .build()
                            .unwrap(),
                        None,
                        move || {
                            boxed(
                                BoxedArgsBuilder::default()
                                    .alignment(Alignment::Center)
                                    .build()
                                    .unwrap(),
                                |scope| {
                                    scope.child(move || {
                                        checkmark(
                                            CheckmarkArgsBuilder::default()
                                                .color(args.checkmark_color)
                                                .stroke_width(args.checkmark_stroke_width)
                                                .progress(progress)
                                                .size(Dp(args.size.0 * 0.8))
                                                .padding([2.0, 2.0])
                                                .build()
                                                .unwrap(),
                                        )
                                    });
                                },
                            );
                        },
                    )
                }
            }
        },
    );
}
