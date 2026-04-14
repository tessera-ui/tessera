//! A customizable, animated checkbox component.
//!
//! ## Usage
//!
//! Use in forms, settings, or lists to enable boolean selections.
use std::time::Duration;

use tessera_ui::{
    CallbackWith, Color, Dp, Modifier, PxSize, RenderSlot, State, accesskit::Role,
    current_frame_nanos, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    checkmark::checkmark,
    modifier::{InteractionState, ModifierExt, PointerEventContext, ToggleableArgs},
    ripple_state::{RippleSpec, RippleState},
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceStyle, surface},
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

    fn default_shape() -> Shape {
        Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(Dp(2.0), 2.0),
            top_right: RoundedCorner::manual(Dp(2.0), 2.0),
            bottom_right: RoundedCorner::manual(Dp(2.0), 2.0),
            bottom_left: RoundedCorner::manual(Dp(2.0), 2.0),
        }
    }

    /// Computes the default state-layer base color for the current checked
    /// state.
    pub fn state_layer_base_color(
        is_checked: bool,
        checked_color: Color,
        scheme: &MaterialColorScheme,
    ) -> Color {
        if is_checked {
            checked_color
        } else {
            scheme.on_surface
        }
    }
}

/// Controller for [`checkbox`] state.
#[derive(Clone, PartialEq, Default)]
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
            self.checkmark.last_toggle_frame_nanos = None;
        }
    }

    /// Toggles the checked state and starts the animation timeline.
    pub fn toggle(&mut self) {
        self.checkmark.toggle();
    }

    /// Advances the checkmark animation progress based on elapsed time.
    fn update_progress(&mut self, frame_nanos: u64) {
        self.checkmark.update_progress(frame_nanos);
    }

    /// Returns current animation progress (0.0..1.0).
    fn progress(&self) -> f32 {
        self.checkmark.progress()
    }

    /// Returns whether the checkmark animation is currently running.
    fn is_animating(&self) -> bool {
        self.checkmark.last_toggle_frame_nanos.is_some()
    }
}

impl CheckboxInnerBuilder {
    fn on_toggle_option_shared(mut self, on_toggle: Option<CallbackWith<bool, ()>>) -> Self {
        self.props.on_toggle = on_toggle;
        self
    }

    fn accessibility_label_option(mut self, accessibility_label: Option<String>) -> Self {
        self.props.accessibility_label = accessibility_label;
        self
    }

    fn accessibility_description_option(
        mut self,
        accessibility_description: Option<String>,
    ) -> Self {
        self.props.accessibility_description = accessibility_description;
        self
    }

    fn controller_option(mut self, controller: Option<State<CheckboxController>>) -> Self {
        self.props.controller = controller;
        self
    }
}

// Animation duration for the checkmark stroke (milliseconds)
const CHECKMARK_ANIMATION_DURATION: Duration = Duration::from_millis(200);

/// State for checkmark animation (similar to `SwitchState`)
#[derive(Clone, PartialEq)]
struct CheckmarkState {
    checked: bool,
    progress: f32,
    last_toggle_frame_nanos: Option<u64>,
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
            last_toggle_frame_nanos: None,
        }
    }

    fn toggle(&mut self) {
        self.checked = !self.checked;
        self.last_toggle_frame_nanos = Some(current_frame_nanos());
    }

    fn update_progress(&mut self, frame_nanos: u64) {
        if let Some(start_frame_nanos) = self.last_toggle_frame_nanos {
            let elapsed_nanos = frame_nanos.saturating_sub(start_frame_nanos);
            let animation_nanos = CHECKMARK_ANIMATION_DURATION
                .as_nanos()
                .min(u64::MAX as u128) as u64;
            let fraction = if animation_nanos == 0 {
                1.0
            } else {
                (elapsed_nanos as f32 / animation_nanos as f32).min(1.0)
            };
            self.progress = if self.checked {
                fraction
            } else {
                1.0 - fraction
            };
            if fraction >= 1.0 {
                self.last_toggle_frame_nanos = None;
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
/// - `modifier` — modifier chain applied to the checkbox subtree.
/// - `on_toggle` — toggle callback invoked with the new checked state.
/// - `checked` — initial checked state.
/// - `size` — optional checkbox glyph size.
/// - `color` — optional unchecked outline color.
/// - `checked_color` — optional checked container color.
/// - `checkmark_color` — optional checkmark icon color.
/// - `checkmark_stroke_width` — optional checkmark stroke width.
/// - `shape` — optional outer checkbox shape.
/// - `disabled` — whether the checkbox is disabled.
/// - `disabled_color` — optional disabled border/container color.
/// - `disabled_checkmark_color` — optional disabled checkmark color.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `controller` — optional external checkbox controller.
///
/// ## Examples
///
/// ```
/// use tessera_components::checkbox::checkbox;
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn checkbox_demo() {
///     let is_checked = remember(|| false);
///     checkbox().checked(true).on_toggle(move |new_value| {
///         is_checked.set(new_value);
///     });
/// }
/// ```
#[tessera]
pub fn checkbox(
    modifier: Option<Modifier>,
    on_toggle: Option<CallbackWith<bool, ()>>,
    checked: Option<bool>,
    size: Option<Dp>,
    color: Option<Color>,
    checked_color: Option<Color>,
    checkmark_color: Option<Color>,
    checkmark_stroke_width: Option<f32>,
    shape: Option<Shape>,
    disabled: Option<bool>,
    disabled_color: Option<Color>,
    disabled_checkmark_color: Option<Color>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    #[prop(skip_setter)] controller: Option<State<CheckboxController>>,
) {
    let modifier = modifier.unwrap_or_default();
    let checked = checked.unwrap_or(false);
    let disabled = disabled.unwrap_or(false);
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let size = size.unwrap_or(CheckboxDefaults::GLYPH_SIZE);
    let color = color.unwrap_or(scheme.on_surface_variant);
    let checked_color = checked_color.unwrap_or(scheme.primary);
    let checkmark_color = checkmark_color.unwrap_or(scheme.on_primary);
    let checkmark_stroke_width = checkmark_stroke_width.unwrap_or(2.5);
    let shape = shape.unwrap_or_else(CheckboxDefaults::default_shape);
    let disabled_color = disabled_color.unwrap_or(
        scheme
            .on_surface
            .with_alpha(MaterialAlpha::DISABLED_CONTENT),
    );
    let disabled_checkmark_color = disabled_checkmark_color.unwrap_or(scheme.surface);
    let controller = controller.unwrap_or_else(|| remember(|| CheckboxController::new(checked)));

    checkbox_inner()
        .modifier(modifier)
        .on_toggle_option_shared(on_toggle)
        .size(size)
        .color(color)
        .checked_color(checked_color)
        .checkmark_color(checkmark_color)
        .checkmark_stroke_width(checkmark_stroke_width)
        .shape(shape)
        .disabled(disabled)
        .disabled_color(disabled_color)
        .disabled_checkmark_color(disabled_checkmark_color)
        .accessibility_label_option(accessibility_label)
        .accessibility_description_option(accessibility_description)
        .controller_option(Some(controller));
}

#[tessera]
fn checkbox_inner(
    modifier: Option<Modifier>,
    on_toggle: Option<CallbackWith<bool, ()>>,
    size: Option<Dp>,
    color: Option<Color>,
    checked_color: Option<Color>,
    checkmark_color: Option<Color>,
    checkmark_stroke_width: Option<f32>,
    shape: Option<Shape>,
    disabled: Option<bool>,
    disabled_color: Option<Color>,
    disabled_checkmark_color: Option<Color>,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    controller: Option<State<CheckboxController>>,
) {
    let modifier = modifier.unwrap_or_default();
    let size = size.unwrap_or(CheckboxDefaults::GLYPH_SIZE);
    let color = color.unwrap_or(Color::TRANSPARENT);
    let checked_color = checked_color.unwrap_or(Color::TRANSPARENT);
    let checkmark_color = checkmark_color.unwrap_or(Color::TRANSPARENT);
    let checkmark_stroke_width = checkmark_stroke_width.unwrap_or(2.5);
    let shape = shape.unwrap_or(Shape::CAPSULE);
    let disabled = disabled.unwrap_or(false);
    let disabled_color = disabled_color.unwrap_or(Color::TRANSPARENT);
    let disabled_checkmark_color = disabled_checkmark_color.unwrap_or(Color::TRANSPARENT);
    let controller = controller.expect("checkbox_inner requires controller to be set");
    if controller.with(|c| c.is_animating()) {
        receive_frame_nanos(move |frame_nanos| {
            let is_animating = controller.with_mut(|controller| {
                controller.update_progress(frame_nanos);
                controller.is_animating()
            });
            if is_animating {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    let is_checked = controller.with(|c| c.is_checked());
    let enabled = !disabled;
    let interaction_state = enabled.then(|| remember(InteractionState::new));
    let ripple_state = enabled.then(|| remember(RippleState::new));
    let on_value_change = {
        let on_toggle = on_toggle.unwrap_or_else(CallbackWith::default_value);
        CallbackWith::new(move |next_checked| {
            controller.with_mut(|c| c.set_checked(next_checked));
            on_toggle.call(next_checked);
        })
    };

    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let (checkbox_style, icon_color) = if disabled {
        if is_checked {
            (
                SurfaceStyle::Filled {
                    color: disabled_color,
                },
                disabled_checkmark_color,
            )
        } else {
            (
                SurfaceStyle::Outlined {
                    color: disabled_color,
                    width: Dp(2.0),
                },
                Color::TRANSPARENT,
            )
        }
    } else if is_checked {
        (
            SurfaceStyle::Filled {
                color: checked_color,
            },
            checkmark_color,
        )
    } else {
        (
            SurfaceStyle::Outlined {
                color,
                width: Dp(2.0),
            },
            Color::TRANSPARENT,
        )
    };

    let state_layer_base =
        CheckboxDefaults::state_layer_base_color(is_checked, checked_color, &scheme);

    let render_checkmark = RenderSlot::new(move || {
        let progress = controller.with(|c| c.progress());
        if progress > 0.0 {
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    checkmark()
                        .color(icon_color)
                        .stroke_width(checkmark_stroke_width)
                        .progress(progress)
                        .size(Dp(size.0 * 0.8))
                        .padding([0.0, 0.0]);
                });
        }
    });

    let render_checkbox_surface = {
        RenderSlot::new(move || {
            let checkbox_style = checkbox_style.clone();
            surface()
                .modifier(Modifier::new().size(size, size))
                .shape(shape)
                .style(checkbox_style)
                .with_child(move || {
                    render_checkmark.render();
                });
        })
    };

    let render_checkbox_container = {
        RenderSlot::new(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    render_checkbox_surface.render();
                });
        })
    };

    let render_state_layer = {
        RenderSlot::new(move || {
            if let Some(state) = interaction_state {
                let mut builder = surface()
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
                    .ripple_color(state_layer_base)
                    .interaction_state(state);
                builder.set_ripple_state(ripple_state);
                builder.with_child(move || {
                    render_checkbox_container.render();
                });
            } else {
                let mut builder = surface()
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
                builder.set_ripple_state(ripple_state);
                builder.with_child(move || {
                    render_checkbox_container.render();
                });
            }
        })
    };

    let mut modifier = modifier.size(
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
            move |ctx: PointerEventContext| {
                state.with_mut(|s| s.start_animation_with_spec(ctx.normalized_pos, size, spec));
            }
        });
        let release_handler = ripple_state
            .map(|state| move |_ctx: PointerEventContext| state.with_mut(|s| s.release()));
        let toggle_args = ToggleableArgs {
            value: is_checked,
            on_value_change,
            enabled: true,
            role: Some(Role::CheckBox),
            label: accessibility_label.clone(),
            description: accessibility_description.clone(),
            interaction_state,
            on_press: press_handler.map(Into::into),
            on_release: release_handler.map(Into::into),
            ..Default::default()
        };
        modifier = modifier.toggleable_with(toggle_args);
    }
    boxed()
        .modifier(modifier)
        .alignment(Alignment::Center)
        .children(move || {
            render_state_layer.render();
        });
}

#[cfg(test)]
mod tests {
    use super::{CHECKMARK_ANIMATION_DURATION, CheckboxController};

    #[test]
    fn checkbox_controller_animates_to_checked_state() {
        let mut controller = CheckboxController::new(false);

        controller.toggle();

        assert!(controller.is_checked());
        assert!(controller.is_animating());
        assert_eq!(controller.progress(), 0.0);

        let half_nanos = (CHECKMARK_ANIMATION_DURATION.as_nanos() / 2) as u64;
        controller.update_progress(half_nanos);
        let mid = controller.progress();
        assert!(mid > 0.0 && mid < 1.0, "mid animation progress was {mid}");
        assert!(controller.is_animating());

        controller.update_progress(CHECKMARK_ANIMATION_DURATION.as_nanos() as u64);
        assert_eq!(controller.progress(), 1.0);
        assert!(!controller.is_animating());
    }

    #[test]
    fn checkbox_controller_animates_to_unchecked_state() {
        let mut controller = CheckboxController::new(true);

        controller.toggle();

        assert!(!controller.is_checked());
        assert!(controller.is_animating());
        assert_eq!(controller.progress(), 1.0);

        let half_nanos = (CHECKMARK_ANIMATION_DURATION.as_nanos() / 2) as u64;
        controller.update_progress(half_nanos);
        let mid = controller.progress();
        assert!(mid > 0.0 && mid < 1.0, "mid animation progress was {mid}");
        assert!(controller.is_animating());

        controller.update_progress(CHECKMARK_ANIMATION_DURATION.as_nanos() as u64);
        assert_eq!(controller.progress(), 0.0);
        assert!(!controller.is_animating());
    }
}
