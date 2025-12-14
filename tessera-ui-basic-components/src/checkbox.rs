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
use tessera_ui::{
    Color, DimensionValue, Dp, State,
    accesskit::{Action, Role, Toggled},
    remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    boxed::{BoxedArgsBuilder, boxed},
    checkmark::{CheckmarkArgsBuilder, checkmark},
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgsBuilder, SurfaceStyle, surface},
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
    /// Default hover alpha for state layers.
    pub const HOVER_ALPHA: f32 = MaterialAlpha::HOVER;
    /// Default pressed alpha for ripple.
    pub const PRESSED_ALPHA: f32 = MaterialAlpha::PRESSED;

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

    /// Derives the hover fill color for the state layer.
    pub fn hover_color(base: Color) -> Color {
        base.with_alpha(Self::HOVER_ALPHA)
    }

    /// Derives the ripple color for the state layer.
    pub fn ripple_color(base: Color) -> Color {
        base.with_alpha(Self::PRESSED_ALPHA)
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
    /// the same value for width and height; default is `Dp(18.0)`.
    #[builder(default = "CheckboxDefaults::GLYPH_SIZE")]
    pub size: Dp,

    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.on_surface_variant")]
    /// Outline color when the checkbox is not checked.
    ///
    /// This sets the border color shown for the unchecked state.
    pub color: Color,

    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.primary")]
    /// Background color used when the checkbox is checked.
    ///
    /// This color is shown behind the checkmark to indicate an active/selected
    /// state. Choose a higher-contrast color relative to `color`.
    pub checked_color: Color,

    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.on_primary")]
    /// Color used to draw the checkmark icon inside the checkbox.
    ///
    /// This is applied on top of the `checked_color` surface.
    pub checkmark_color: Color,

    #[builder(default = "2.5")]
    /// Stroke width in physical pixels used to render the checkmark path.
    ///
    /// Higher values produce a thicker checkmark. The default value is tuned
    /// for the default `size`.
    pub checkmark_stroke_width: f32,

    #[builder(
        default = "Shape::RoundedRectangle{ top_left: RoundedCorner::manual(Dp(2.0), 2.0), top_right: RoundedCorner::manual(Dp(2.0), 2.0), bottom_right: RoundedCorner::manual(Dp(2.0), 2.0), bottom_left: RoundedCorner::manual(Dp(2.0), 2.0) }"
    )]
    /// Shape used for the outer checkbox surface (rounded rectangle, etc.).
    ///
    /// Use this to customize the corner radii or switch to alternate shapes.
    pub shape: Shape,

    /// Optional surface color to apply when the pointer hovers over the
    /// control.
    ///
    /// If `None`, the control uses the default Material 3 state layer behavior.
    #[builder(default)]
    pub hover_color: Option<Color>,

    /// Whether the checkbox is disabled.
    #[builder(default = "false")]
    pub disabled: bool,

    #[builder(
        default = "use_context::<MaterialTheme>().get().color_scheme.on_surface.with_alpha(MaterialAlpha::DISABLED_CONTENT)"
    )]
    /// Color used for the checkbox border/background when disabled.
    pub disabled_color: Color,

    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.surface")]
    /// Color used for the checkmark icon when disabled.
    pub disabled_checkmark_color: Color,

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
/// use std::sync::Arc;
/// use tessera_ui::{Dp, remember, tessera};
/// use tessera_ui_basic_components::checkbox::{CheckboxArgsBuilder, checkbox};
///
/// // A tiny UI demo that shows a checkbox and a text label that reflects its state.
/// #[tessera]
/// fn checkbox_demo() {
///     let is_checked = remember(|| false);
///     let on_toggle = Arc::new(move |new_value| is_checked.set(new_value));
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
/// use tessera_ui::{Dp, remember, tessera};
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
    controller: State<CheckboxController>,
) {
    let args: CheckboxArgs = args.into();
    let enabled = !args.disabled;

    // Clone fields needed for closures before moving on_toggle
    let size = args.size;
    let shape = args.shape;

    // Click handler: toggle animation state and forward toggle callback
    let on_click = if enabled {
        let on_toggle = args.on_toggle.clone();
        Some(Arc::new(move || {
            controller.with_mut(|c| c.toggle());
            on_toggle(controller.with(|c| c.is_checked()));
        }))
    } else {
        None
    };
    let on_click_for_surface = on_click.clone();

    // Determine colors based on state
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
    let is_checked = controller.with(|c| c.is_checked());
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

    // State layer colors (hover + ripple) follow Material 3.
    let state_layer_base = CheckboxDefaults::state_layer_base_color(is_checked, &args, &scheme);
    let state_layer_hover_color = if enabled {
        args.hover_color
            .unwrap_or_else(|| CheckboxDefaults::hover_color(state_layer_base))
    } else {
        Color::TRANSPARENT
    };
    let state_layer_ripple_color = CheckboxDefaults::ripple_color(state_layer_base);

    // Checkmark
    let checkmark_stroke_width = args.checkmark_stroke_width;
    let checkbox_size = args.size;
    let render_checkmark = move || {
        let progress = controller.with(|c| c.progress());
        if progress > 0.0 {
            boxed(
                BoxedArgsBuilder::default()
                    .alignment(Alignment::Center)
                    .width(DimensionValue::FILLED)
                    .height(DimensionValue::FILLED)
                    .build()
                    .expect("builder construction failed"),
                |scope| {
                    scope.child(move || {
                        checkmark(
                            CheckmarkArgsBuilder::default()
                                .color(icon_color)
                                .stroke_width(checkmark_stroke_width)
                                .progress(progress)
                                .size(Dp(checkbox_size.0 * 0.8))
                                .padding([0.0, 0.0])
                                .build()
                                .expect("builder construction failed"),
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
                SurfaceArgsBuilder::default()
                    .width(DimensionValue::Fixed(size.to_px()))
                    .height(DimensionValue::Fixed(size.to_px()))
                    .shape(shape)
                    .style(checkbox_style)
                    .build()
                    .expect("builder construction failed"),
                render_checkmark,
            );
        }
    );

    // Checkbox Container (centering the 18x18 surface)
    let render_checkbox_container = closure!(
        clone render_checkbox_surface,
        || {
            boxed(
                BoxedArgsBuilder::default()
                    .alignment(Alignment::Center)
                    .width(DimensionValue::FILLED)
                    .height(DimensionValue::FILLED)
                    .build()
                    .expect("builder construction failed"),
                |scope| {
                    scope.child(render_checkbox_surface);
                },
            );
        }
    );

    // State Layer Surface (40x40)
    let render_state_layer = closure!(
        clone enabled,
        clone state_layer_hover_color,
        clone state_layer_ripple_color,
        clone on_click_for_surface,
        clone render_checkbox_container,
        || {
            let mut builder = SurfaceArgsBuilder::default()
                .width(DimensionValue::Fixed(CheckboxDefaults::STATE_LAYER_SIZE.to_px()))
                .height(DimensionValue::Fixed(CheckboxDefaults::STATE_LAYER_SIZE.to_px()))
                .shape(Shape::Ellipse)
                .enabled(enabled)
                .style(SurfaceStyle::Filled {
                    color: Color::TRANSPARENT,
                })
                .hover_style(enabled.then_some(SurfaceStyle::Filled {
                    color: state_layer_hover_color,
                }))
                .ripple_color(state_layer_ripple_color);

            if let Some(handler) = on_click_for_surface.clone() {
                builder = builder.on_click_shared(handler);
            }

            surface(
                builder.build().expect("builder construction failed"),
                render_checkbox_container,
            );
        }
    );

    // Outer Box (Layout 48x48)
    boxed(
        BoxedArgsBuilder::default()
            .width(DimensionValue::Fixed(
                CheckboxDefaults::TOUCH_TARGET_SIZE.to_px(),
            ))
            .height(DimensionValue::Fixed(
                CheckboxDefaults::TOUCH_TARGET_SIZE.to_px(),
            ))
            .alignment(Alignment::Center)
            .build()
            .expect("builder construction failed"),
        closure!(
            clone render_state_layer,
            |scope| {
                scope.child(render_state_layer);
            }
        ),
    );

    let accessibility_label = args.accessibility_label.clone();
    let accessibility_description = args.accessibility_description.clone();
    let on_click_for_accessibility = on_click.clone();
    let disabled = args.disabled;
    input_handler(Box::new(move |input| {
        // Update animation progress
        controller.with_mut(|c| c.update_progress());

        let checked = controller.with(|c| c.is_checked());
        let mut builder = input.accessibility().role(Role::CheckBox);

        if let Some(label) = &accessibility_label {
            builder = builder.label(label.clone());
        }
        if let Some(description) = &accessibility_description {
            builder = builder.description(description.clone());
        }

        if disabled {
            builder = builder.disabled();
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

        if !disabled && let Some(handler) = on_click_for_accessibility.as_ref() {
            let handler = handler.clone();
            input.set_accessibility_action_handler(move |action| {
                if action == Action::Click {
                    handler();
                }
            });
        }
    }));
}
