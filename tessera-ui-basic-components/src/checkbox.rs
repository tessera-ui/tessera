//! A customizable, animated checkbox component.
//!
//! ## Usage
//!
//! Use in forms, settings, or lists to enable boolean selections.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{
    Color, DimensionValue, Dp,
    accesskit::{Action, Role, Toggled},
    tessera,
};

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
    ripple: RippleState,
    checkmark: Arc<RwLock<CheckmarkState>>,
}

impl CheckboxState {
    pub fn new(initial_state: bool) -> Self {
        Self {
            ripple: RippleState::new(),
            checkmark: Arc::new(RwLock::new(CheckmarkState::new(initial_state))),
        }
    }
}

/// Arguments for the `checkbox` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct CheckboxArgs {
    /// Callback invoked when the checkbox is toggled.
    #[builder(default = "Arc::new(|_| {})")]
    pub on_toggle: Arc<dyn Fn(bool) + Send + Sync>,
    /// Size of the checkbox (width and height).
    ///
    /// Expressed in `Dp` (density-independent pixels). The checkbox will use
    /// the same value for width and height; default is `Dp(24.0)`.
    #[builder(default = "Dp(24.0)")]
    pub size: Dp,

    #[builder(default = "Color::new(0.8, 0.8, 0.8, 1.0)")]
    /// Background color when the checkbox is not checked.
    ///
    /// This sets the surface color shown for the unchecked state and is typically
    /// a subtle neutral color.
    pub color: Color,

    #[builder(default = "Color::new(0.6, 0.7, 0.9, 1.0)")]
    /// Background color used when the checkbox is checked.
    ///
    /// This color is shown behind the checkmark to indicate an active/selected
    /// state. Choose a higher-contrast color relative to `color`.
    pub checked_color: Color,

    #[builder(default = "Color::from_rgb_u8(119, 72, 146)")]
    /// Color used to draw the checkmark icon inside the checkbox.
    ///
    /// This is applied on top of the `checked_color` surface.
    pub checkmark_color: Color,

    #[builder(default = "5.0")]
    /// Stroke width in physical pixels used to render the checkmark path.
    ///
    /// Higher values produce a thicker checkmark. The default value is tuned for
    /// the default `size`.
    pub checkmark_stroke_width: f32,

    #[builder(default = "1.0")]
    /// Initial animation progress of the checkmark (0.0 ..= 1.0).
    ///
    /// Used to drive the checkmark animation when toggling. `0.0` means not
    /// visible; `1.0` means fully drawn. Values in-between show the intermediate
    /// animation state.
    pub checkmark_animation_progress: f32,

    #[builder(
        default = "Shape::RoundedRectangle{ top_left: Dp(4.0), top_right: Dp(4.0), bottom_right: Dp(4.0), bottom_left: Dp(4.0), g2_k_value: 3.0 }"
    )]
    pub shape: Shape,
    /// Shape used for the outer checkbox surface (rounded rectangle, etc.).
    ///
    /// Use this to customize the corner radii or switch to alternate shapes.

    #[builder(default)]
    pub hover_color: Option<Color>,
    /// Optional surface color to apply when the pointer hovers over the control.
    ///
    /// If `None`, the control does not apply a hover style by default.
    /// Optional accessibility label read by assistive technologies.
    ///
    /// The label should be a short, human-readable string describing the
    /// purpose of the checkbox (for example "Enable auto-save").
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description read by assistive technologies.
    ///
    /// A longer description or contextual helper text that augments the
    /// `accessibility_label` for users of assistive technology.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
}

impl Default for CheckboxArgs {
    fn default() -> Self {
        CheckboxArgsBuilder::default()
            .build()
            .expect("CheckboxArgsBuilder default build should succeed")
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

/// # checkbox
///
/// Renders an interactive checkbox with an animated checkmark.
///
/// ## Usage
///
/// Use to capture a boolean (true/false) choice from the user.
///
/// ## Parameters
///
/// - `args` — configures the checkbox's appearance and `on_toggle` callback; see [`CheckboxArgs`].
/// - `state` — a clonable [`CheckboxState`] that manages the checkmark and ripple animations.
///
/// ## Examples
///
/// ```
/// use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
/// use tessera_ui::{tessera, Color, Dp};
/// use tessera_ui_basic_components::checkbox::{checkbox, CheckboxArgsBuilder, CheckboxState};
///
/// // A tiny UI demo that shows a checkbox and a text label that reflects its state.
/// #[derive(Clone, Default)]
/// struct DemoState {
///     is_checked: Arc<AtomicBool>,
///     checkbox_state: CheckboxState,
/// }
///
/// #[tessera]
/// fn checkbox_demo(state: DemoState) {
///     // Build a simple checkbox whose on_toggle updates `is_checked`.
///     let on_toggle = Arc::new({
///         let is_checked = state.is_checked.clone();
///         move |new_value| {
///             is_checked.store(new_value, Ordering::SeqCst);
///         }
///     });
///
///     // Render the checkbox; the example shows a minimal pattern for interactive demos.
///     checkbox(
///         CheckboxArgsBuilder::default()
///             .on_toggle(on_toggle)
///             .build()
///             .unwrap(),
///         state.checkbox_state.clone(),
///     );
/// }
/// ```
#[tessera]
pub fn checkbox(args: impl Into<CheckboxArgs>, state: CheckboxState) {
    let args: CheckboxArgs = args.into();

    // If a state is provided, set up an updater to advance the animation each frame
    let checkmark_state = state.checkmark.clone();
    input_handler(Box::new(move |_input| {
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
    let on_click_for_surface = on_click.clone();

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
            .on_click(on_click_for_surface)
            .build().expect("builder construction failed"),
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
                            .build().expect("builder construction failed"),
                        None,
                        move || {
                            boxed(
                                BoxedArgsBuilder::default()
                                    .alignment(Alignment::Center)
                                    .build().expect("builder construction failed"),
                                |scope| {
                                    scope.child(move || {
                                        checkmark(
                                            CheckmarkArgsBuilder::default()
                                                .color(args.checkmark_color)
                                                .stroke_width(args.checkmark_stroke_width)
                                                .progress(progress)
                                                .size(Dp(args.size.0 * 0.8))
                                                .padding([2.0, 2.0])
                                                .build().expect("builder construction failed"),
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

    let accessibility_label = args.accessibility_label.clone();
    let accessibility_description = args.accessibility_description.clone();
    let accessibility_state = state.clone();
    let on_click_for_accessibility = on_click.clone();
    input_handler(Box::new(move |input| {
        let checked = accessibility_state.checkmark.read().checked;
        let mut builder = input.accessibility().role(Role::CheckBox);

        if let Some(label) = accessibility_label.as_ref() {
            builder = builder.label(label.clone());
        }
        if let Some(description) = accessibility_description.as_ref() {
            builder = builder.description(description.clone());
        }

        builder = builder
            .focusable()
            .action(Action::Click)
            .toggled(if checked {
                Toggled::True
            } else {
                Toggled::False
            });

        builder.commit();

        input.set_accessibility_action_handler({
            let on_click = on_click_for_accessibility.clone();
            move |action| {
                if action == Action::Click {
                    on_click();
                }
            }
        });
    }));
}


