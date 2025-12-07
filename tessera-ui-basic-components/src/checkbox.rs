//! A customizable, animated checkbox component.
//!
//! ## Usage
//!
//! Use in forms, settings, or lists to enable boolean selections.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use closure::closure;
use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{
    Color, DimensionValue, Dp,
    accesskit::{Action, Role, Toggled},
    remember, tessera,
};

use crate::{
    alignment::Alignment,
    boxed::{BoxedArgsBuilder, boxed},
    checkmark::{CheckmarkArgsBuilder, checkmark},
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgsBuilder, surface},
};

/// Controller for [`checkbox`] state.
pub struct CheckboxController {
    checkmark: RwLock<CheckmarkState>,
}

impl CheckboxController {
    /// Creates a new controller with the provided initial checked state.
    pub fn new(initial_state: bool) -> CheckboxController {
        Self {
            checkmark: RwLock::new(CheckmarkState::new(initial_state)),
        }
    }

    /// Returns whether the checkbox is currently checked.
    pub fn is_checked(&self) -> bool {
        self.checkmark.read().checked
    }

    /// Sets the checked state directly and resets animation progress.
    pub fn set_checked(&self, checked: bool) {
        let mut state = self.checkmark.write();
        if state.checked != checked {
            state.checked = checked;
            state.progress = if checked { 1.0 } else { 0.0 };
            state.last_toggle_time = None;
        }
    }

    /// Toggles the checked state and starts the animation timeline.
    pub fn toggle(&self) {
        self.checkmark.write().toggle();
    }

    /// Advances the checkmark animation progress based on elapsed time.
    fn update_progress(&self) {
        self.checkmark.write().update_progress();
    }

    /// Returns current animation progress (0.0..1.0).
    fn progress(&self) -> f32 {
        self.checkmark.read().progress()
    }
}

impl Default for CheckboxController {
    fn default() -> Self {
        Self {
            checkmark: RwLock::new(CheckmarkState::default()),
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
    /// Initial checked state for the checkbox.
    #[builder(default = "false")]
    pub checked: bool,
    /// Size of the checkbox (width and height).
    ///
    /// Expressed in `Dp` (density-independent pixels). The checkbox will use
    /// the same value for width and height; default is `Dp(24.0)`.
    #[builder(default = "Dp(24.0)")]
    pub size: Dp,

    #[builder(default = "crate::material_color::global_material_scheme().surface_variant")]
    /// Background color when the checkbox is not checked.
    ///
    /// This sets the surface color shown for the unchecked state and is typically
    /// a subtle neutral color.
    pub color: Color,

    #[builder(default = "crate::material_color::global_material_scheme().primary")]
    /// Background color used when the checkbox is checked.
    ///
    /// This color is shown behind the checkmark to indicate an active/selected
    /// state. Choose a higher-contrast color relative to `color`.
    pub checked_color: Color,

    #[builder(default = "crate::material_color::global_material_scheme().on_primary")]
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
        default = "Shape::RoundedRectangle{ top_left: RoundedCorner::manual(Dp(4.0), 3.0), top_right: RoundedCorner::manual(Dp(4.0), 3.0), bottom_right: RoundedCorner::manual(Dp(4.0), 3.0), bottom_left: RoundedCorner::manual(Dp(4.0), 3.0) }"
    )]
    /// Shape used for the outer checkbox surface (rounded rectangle, etc.).
    ///
    /// Use this to customize the corner radii or switch to alternate shapes.
    pub shape: Shape,

    /// Optional surface color to apply when the pointer hovers over the control.
    ///
    /// If `None`, the control does not apply a hover style by default.
    #[builder(
        default = "Some(crate::material_color::blend_over(crate::material_color::global_material_scheme().surface_variant, crate::material_color::global_material_scheme().on_surface, 0.08))"
    )]
    pub hover_color: Option<Color>,

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
struct CheckmarkState {
    checked: bool,
    progress: f32,
    last_toggle_time: Option<Instant>,
}

impl Default for CheckmarkState {
    fn default() -> Self {
        Self::new(false)
    }
}

impl CheckmarkState {
    fn new(initial_state: bool) -> Self {
        Self {
            checked: initial_state,
            progress: if initial_state { 1.0 } else { 0.0 },
            last_toggle_time: None,
        }
    }

    /// Toggle checked state and start animation
    fn toggle(&mut self) {
        self.checked = !self.checked;
        self.last_toggle_time = Some(Instant::now());
    }

    /// Update progress based on elapsed time
    fn update_progress(&mut self) {
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

    fn progress(&self) -> f32 {
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
/// - `args` — configures the checkbox's appearance, initial state, and `on_toggle` callback; see [`CheckboxArgs`].
/// - `controller` — optional external controller; use [`checkbox_with_controller`] for a controlled checkbox.
///
/// ## Examples
///
/// ```
/// use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
/// use tessera_ui::{tessera, Color, Dp};
/// use tessera_ui_basic_components::checkbox::{
///     checkbox, CheckboxArgsBuilder,
/// };
///
/// // A tiny UI demo that shows a checkbox and a text label that reflects its state.
/// #[derive(Clone, Default)]
/// struct DemoState {
///     is_checked: Arc<AtomicBool>,
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
///     checkbox(
///         CheckboxArgsBuilder::default()
///             .checked(true)
///             .on_toggle(on_toggle)
///             .build()
///             .unwrap(),
///     );
/// }
/// ```
#[tessera]
pub fn checkbox(args: impl Into<CheckboxArgs>) {
    let args: CheckboxArgs = args.into();
    let controller = remember(|| CheckboxController::new(args.checked));
    checkbox_with_controller(args, controller);
}

/// # checkbox_with_controller
///
/// Controlled checkbox variant that accepts an explicit controller.
///
/// ## Usage
///
/// Use when you need to drive or observe the checked state from outside the component.
///
/// ## Parameters
///
/// - `args` — configures the checkbox appearance and callbacks; see [`CheckboxArgs`].
/// - `controller` — a [`CheckboxController`] that owns the checked state and animation timeline.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{tessera, Dp, remember};
/// use tessera_ui_basic_components::checkbox::{
///     CheckboxArgsBuilder, CheckboxController, checkbox_with_controller,
/// };
///
/// #[tessera]
/// fn controlled_demo() {
///     let controller = remember(|| CheckboxController::new(false));
///     checkbox_with_controller(
///         CheckboxArgsBuilder::default()
///             .size(Dp(20.0))
///             .build()
///             .unwrap(),
///         controller,
///     );
/// }
/// ```
#[tessera]
pub fn checkbox_with_controller(
    args: impl Into<CheckboxArgs>,
    controller: Arc<CheckboxController>,
) {
    let args: CheckboxArgs = args.into();

    // Advance the animation each frame
    input_handler(Box::new({
        let controller = controller.clone();
        move |_input| {
            controller.update_progress();
        }
    }));

    // Click handler: toggle animation state and forward toggle callback
    let on_click = Arc::new(closure!(clone controller, clone args.on_toggle, || {
        controller.toggle();
        on_toggle(controller.is_checked());
    }));
    let on_click_for_surface = on_click.clone();

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(args.size.to_px()))
            .height(DimensionValue::Fixed(args.size.to_px()))
            .style(
                if controller.is_checked() {
                    args.checked_color
                } else {
                    args.color
                }
                .into(),
            )
            .hover_style(args.hover_color.map(|c| c.into()))
            .shape(args.shape)
            .on_click(on_click_for_surface)
            .build()
            .expect("builder construction failed"),
        closure!(
            clone controller,
            clone args.checkmark_color,
            clone args.checkmark_stroke_width,
            clone args.size,
            || {
            let progress = controller.progress();
            if progress > 0.0 {
                surface(
                    SurfaceArgsBuilder::default()
                        .padding(Dp(2.0))
                        .style(Color::TRANSPARENT.into())
                        .build()
                        .expect("builder construction failed"),
                    move || {
                        boxed(
                            BoxedArgsBuilder::default()
                                .alignment(Alignment::Center)
                                .build()
                                .expect("builder construction failed"),
                            |scope| {
                                scope.child(move || {
                                    checkmark(
                                        CheckmarkArgsBuilder::default()
                                            .color(checkmark_color)
                                            .stroke_width(checkmark_stroke_width)
                                            .progress(progress)
                                            .size(Dp(size.0 * 0.8))
                                            .padding([2.0, 2.0])
                                            .build()
                                            .expect("builder construction failed"),
                                    )
                                });
                            },
                        );
                    },
                )
            }
        }
        ),
    );

    let accessibility_label = args.accessibility_label.clone();
    let accessibility_description = args.accessibility_description.clone();
    let accessibility_state = controller.clone();
    let on_click_for_accessibility = on_click.clone();
    input_handler(Box::new(closure!(
        clone accessibility_state,
        clone accessibility_label,
        clone accessibility_description,
        clone on_click_for_accessibility,
        |input| {
            let checked = accessibility_state.is_checked();
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

            input.set_accessibility_action_handler(closure!(
                clone on_click_for_accessibility,
                |action| {
                    if action == Action::Click {
                        on_click_for_accessibility();
                    }
                }
            ));
        }
    )));
}
