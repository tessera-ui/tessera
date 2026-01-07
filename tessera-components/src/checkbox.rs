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
use derive_setters::Setters;
use tessera_ui::{
    Color, Dp, Modifier, PxSize, State, accesskit::Role, remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    boxed::{BoxedArgs, boxed},
    checkmark::{CheckmarkArgs, checkmark},
    modifier::{InteractionState, ModifierExt, PointerEventContext, ToggleableArgs},
    ripple_state::{RippleSpec, RippleState},
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgs, SurfaceStyle, surface},
    theme::{MaterialAlpha, MaterialColorScheme, MaterialTheme},
};

/// Material Design 3 defaults for [`checkbox`].
pub struct CheckboxDefaults;

impl CheckboxDefaults {
    /// Visual checkbox glyph size (not including touch target).
    pub const GLYPH_SIZE: Dp = Dp(18.0);
    /// State-layer size used for hover/press feedback.
    pub const STATE_LAYER_SIZE: Dp = Dp(40.0);
    /// Minimum recommended touch target size.
    pub const TOUCH_TARGET_SIZE: Dp = Dp(48.0);
    /// Computes the default state-layer base color for the current checked
    /// state.
    pub fn state_layer_base_color(
        is_checked: bool,
        args: &CheckboxArgs,
        scheme: &MaterialColorScheme,
    ) -> Color {
        if is_checked {
            args.checked_color
        } else {
            scheme.on_surface
        }
    }
}

/// Controller for [`checkbox`] state.
#[derive(Clone, Default)]
pub struct CheckboxController {
    checkmark: CheckmarkState,
}

impl CheckboxController {
    /// Creates a new controller with the provided initial checked state.
    pub fn new(initial_state: bool) -> CheckboxController {
        Self {
            checkmark: CheckmarkState::new(initial_state),
        }
    }

    /// Returns whether the checkbox is currently checked.
    pub fn is_checked(&self) -> bool {
        self.checkmark.checked
    }

    /// Sets the checked state directly and resets animation progress.
    pub fn set_checked(&mut self, checked: bool) {
        if self.checkmark.checked != checked {
            self.checkmark.checked = checked;
            self.checkmark.progress = if checked { 1.0 } else { 0.0 };
            self.checkmark.last_toggle_time = None;
        }
    }

    /// Toggles the checked state and starts the animation timeline.
    pub fn toggle(&mut self) {
        self.checkmark.toggle();
    }

    /// Advances the checkmark animation progress based on elapsed time.
    fn update_progress(&mut self) {
        self.checkmark.update_progress();
    }

    /// Returns current animation progress (0.0..1.0).
    fn progress(&self) -> f32 {
        self.checkmark.progress()
    }
}

/// Arguments for the `checkbox` component.
#[derive(Clone, Setters)]
pub struct CheckboxArgs {
    /// Optional modifier chain applied to the checkbox subtree.
    pub modifier: Modifier,
    /// Callback invoked when the checkbox is toggled.
    #[setters(skip)]
    pub on_toggle: Arc<dyn Fn(bool) + Send + Sync>,
    /// Initial checked state for the checkbox.
    pub checked: bool,
    /// Size of the checkbox (width and height).
    ///
    /// Expressed in `Dp` (density-independent pixels). The checkbox will use
    /// the same value for width and height; default is `Dp(18.0)`.
    pub size: Dp,

    /// Outline color when the checkbox is not checked.
    ///
    /// This sets the border color shown for the unchecked state.
    pub color: Color,

    /// Background color used when the checkbox is checked.
    ///
    /// This color is shown behind the checkmark to indicate an active/selected
    /// state. Choose a higher-contrast color relative to `color`.
    pub checked_color: Color,

    /// Color used to draw the checkmark icon inside the checkbox.
    ///
    /// This is applied on top of the `checked_color` surface.
    pub checkmark_color: Color,

    /// Stroke width in physical pixels used to render the checkmark path.
    ///
    /// Higher values produce a thicker checkmark. The default value is tuned
    /// for the default `size`.
    pub checkmark_stroke_width: f32,

    /// Shape used for the outer checkbox surface (rounded rectangle, etc.).
    ///
    /// Use this to customize the corner radii or switch to alternate shapes.
    pub shape: Shape,

    /// Whether the checkbox is disabled.
    pub disabled: bool,

    /// Color used for the checkbox border/background when disabled.
    pub disabled_color: Color,

    /// Color used for the checkmark icon when disabled.
    pub disabled_checkmark_color: Color,

    /// Optional accessibility label read by assistive technologies.
    ///
    /// The label should be a short, human-readable string describing the
    /// purpose of the checkbox (for example "Enable auto-save").
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description read by assistive technologies.
    ///
    /// A longer description or contextual helper text that augments the
    /// `accessibility_label` for users of assistive technology.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
}

impl CheckboxArgs {
    /// Sets the on_toggle handler.
    pub fn on_toggle<F>(mut self, on_toggle: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_toggle = Arc::new(on_toggle);
        self
    }

    /// Sets the on_toggle handler using a shared callback.
    pub fn on_toggle_shared(mut self, on_toggle: Arc<dyn Fn(bool) + Send + Sync>) -> Self {
        self.on_toggle = on_toggle;
        self
    }
}

impl Default for CheckboxArgs {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        Self {
            modifier: Modifier::new(),
            on_toggle: Arc::new(|_| {}),
            checked: false,
            size: CheckboxDefaults::GLYPH_SIZE,
            color: scheme.on_surface_variant,
            checked_color: scheme.primary,
            checkmark_color: scheme.on_primary,
            checkmark_stroke_width: 2.5,
            shape: Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(2.0), 2.0),
                top_right: RoundedCorner::manual(Dp(2.0), 2.0),
                bottom_right: RoundedCorner::manual(Dp(2.0), 2.0),
                bottom_left: RoundedCorner::manual(Dp(2.0), 2.0),
            },
            disabled: false,
            disabled_color: scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTENT),
            disabled_checkmark_color: scheme.surface,
            accessibility_label: None,
            accessibility_description: None,
        }
    }
}

// Animation duration for the checkmark stroke (milliseconds)
const CHECKMARK_ANIMATION_DURATION: Duration = Duration::from_millis(200);

/// State for checkmark animation (similar to `SwitchState`)
#[derive(Clone)]
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
/// - `args` — configures the checkbox's appearance, initial state, and
///   `on_toggle` callback; see [`CheckboxArgs`].
/// - `controller` — optional external controller; use
///   [`checkbox_with_controller`] for a controlled checkbox.
///
/// ## Examples
///
/// ```
/// use tessera_components::checkbox::{CheckboxArgs, checkbox};
/// use tessera_ui::{Dp, remember, tessera};
///
/// // A tiny UI demo that shows a checkbox and a text label that reflects its state.
/// #[tessera]
/// fn checkbox_demo() {
///     let is_checked = remember(|| false);
///     checkbox(
///         CheckboxArgs::default()
///             .checked(true)
///             .on_toggle(move |new_value| {
///                 is_checked.set(new_value);
///             }),
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
/// Use when you need to drive or observe the checked state from outside the
/// component.
///
/// ## Parameters
///
/// - `args` — configures the checkbox appearance and callbacks; see
///   [`CheckboxArgs`].
/// - `controller` — a [`CheckboxController`] that owns the checked state and
///   animation timeline.
///
/// ## Examples
///
/// ```
/// use tessera_components::checkbox::{
///     CheckboxArgs, CheckboxController, checkbox_with_controller,
/// };
/// use tessera_ui::{Dp, remember, tessera};
///
/// #[tessera]
/// fn controlled_demo() {
///     let controller = remember(|| CheckboxController::new(false));
///     checkbox_with_controller(CheckboxArgs::default().size(Dp(20.0)), controller);
/// }
/// ```
#[tessera]
pub fn checkbox_with_controller(
    args: impl Into<CheckboxArgs>,
    controller: State<CheckboxController>,
) {
    let args: CheckboxArgs = args.into();
    let enabled = !args.disabled;
    controller.with_mut(|c| c.update_progress());

    // Clone fields needed for closures before moving on_toggle
    let size = args.size;
    let shape = args.shape;

    let is_checked = controller.with(|c| c.is_checked());
    let interaction_state = enabled.then(|| remember(InteractionState::new));
    let ripple_state = enabled.then(|| remember(RippleState::new));
    let on_value_change = {
        let on_toggle = args.on_toggle.clone();
        Arc::new(move |checked| {
            controller.with_mut(|c| c.set_checked(checked));
            on_toggle(checked);
        }) as Arc<dyn Fn(bool) + Send + Sync>
    };

    // Determine colors based on state
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let (checkbox_style, icon_color) = if args.disabled {
        if is_checked {
            (
                SurfaceStyle::Filled {
                    color: args.disabled_color,
                },
                args.disabled_checkmark_color,
            )
        } else {
            (
                SurfaceStyle::Outlined {
                    color: args.disabled_color,
                    width: Dp(2.0),
                },
                Color::TRANSPARENT,
            )
        }
    } else if is_checked {
        (
            SurfaceStyle::Filled {
                color: args.checked_color,
            },
            args.checkmark_color,
        )
    } else {
        (
            SurfaceStyle::Outlined {
                color: args.color,
                width: Dp(2.0),
            },
            Color::TRANSPARENT,
        )
    };

    let state_layer_base = CheckboxDefaults::state_layer_base_color(is_checked, &args, &scheme);

    // Checkmark
    let checkmark_stroke_width = args.checkmark_stroke_width;
    let checkbox_size = args.size;
    let render_checkmark = move || {
        let progress = controller.with(|c| c.progress());
        if progress > 0.0 {
            boxed(
                BoxedArgs::default()
                    .alignment(Alignment::Center)
                    .modifier(Modifier::new().fill_max_size()),
                |scope| {
                    scope.child(move || {
                        checkmark(
                            CheckmarkArgs::default()
                                .color(icon_color)
                                .stroke_width(checkmark_stroke_width)
                                .progress(progress)
                                .size(Dp(checkbox_size.0 * 0.8))
                                .padding([0.0, 0.0]),
                        )
                    });
                },
            );
        }
    };

    // Checkbox Surface (18x18)
    let render_checkbox_surface = closure!(
        clone size,
        clone shape,
        clone checkbox_style,
        clone render_checkmark,
        || {
            surface(
                SurfaceArgs::default()
                    .modifier(Modifier::new().size(size, size))
                    .shape(shape)
                    .style(checkbox_style),
                render_checkmark,
            );
        }
    );

    // Checkbox Container (centering the 18x18 surface)
    let render_checkbox_container = closure!(
        clone render_checkbox_surface,
        || {
            boxed(
                BoxedArgs::default()
                    .alignment(Alignment::Center)
                    .modifier(Modifier::new().fill_max_size()),
                |scope| {
                    scope.child(render_checkbox_surface);
                },
            );
        }
    );

    // State Layer Surface (40x40)
    let render_state_layer = closure!(
        clone enabled,
        clone state_layer_base,
        clone interaction_state,
        clone render_checkbox_container,
        clone ripple_state,
        || {
            let mut surface_args = SurfaceArgs::default()
                .modifier(Modifier::new().size(
                    CheckboxDefaults::STATE_LAYER_SIZE,
                    CheckboxDefaults::STATE_LAYER_SIZE,
                ))
                .shape(Shape::Ellipse)
                .enabled(enabled)
                .style(SurfaceStyle::Filled {
                    color: Color::TRANSPARENT,
                })
                .ripple_bounded(false)
                .ripple_radius(Dp(CheckboxDefaults::STATE_LAYER_SIZE.0 / 2.0))
                .ripple_color(state_layer_base);

            if let Some(state) = interaction_state {
                surface_args = surface_args.interaction_state(state);
            }

            let mut surface_args = surface_args;
            surface_args.set_ripple_state(ripple_state);

            surface(surface_args, render_checkbox_container);
        }
    );

    // Outer Box (Layout 48x48)
    let mut modifier = args.modifier.size(
        CheckboxDefaults::TOUCH_TARGET_SIZE,
        CheckboxDefaults::TOUCH_TARGET_SIZE,
    );
    if enabled {
        let ripple_spec = RippleSpec {
            bounded: false,
            radius: Some(Dp(CheckboxDefaults::STATE_LAYER_SIZE.0 / 2.0)),
        };
        let ripple_size = PxSize::new(
            CheckboxDefaults::STATE_LAYER_SIZE.to_px(),
            CheckboxDefaults::STATE_LAYER_SIZE.to_px(),
        );
        let press_handler = ripple_state.map(|state| {
            let spec = ripple_spec;
            let size = ripple_size;
            Arc::new(move |ctx: PointerEventContext| {
                state.with_mut(|s| s.start_animation_with_spec(ctx.normalized_pos, size, spec));
            })
        });
        let release_handler = ripple_state.map(|state| {
            Arc::new(move |_ctx: PointerEventContext| state.with_mut(|s| s.release()))
        });
        let mut toggle_args = ToggleableArgs::new(is_checked, on_value_change)
            .enabled(true)
            .role(Role::CheckBox);
        if let Some(label) = args.accessibility_label.clone() {
            toggle_args = toggle_args.label(label);
        }
        if let Some(desc) = args.accessibility_description.clone() {
            toggle_args = toggle_args.description(desc);
        }
        if let Some(state) = interaction_state {
            toggle_args = toggle_args.interaction_state(state);
        }
        if let Some(handler) = press_handler {
            toggle_args = toggle_args.on_press(handler);
        }
        if let Some(handler) = release_handler {
            toggle_args = toggle_args.on_release(handler);
        }
        modifier = modifier.toggleable(toggle_args);
    }
    boxed(
        BoxedArgs::default()
            .modifier(modifier)
            .alignment(Alignment::Center),
        closure!(
            clone render_state_layer,
            |scope| {
                scope.child(render_state_layer);
            }
        ),
    );
}
