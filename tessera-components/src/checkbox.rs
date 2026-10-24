//! A customizable, animated checkbox component.
//!
//! ## Usage
//!
//! Use in forms, settings, or lists to enable boolean selections.
use tessera_foundation::gesture::TapRecognizer;
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, CallbackWith, Color, Dp, FocusRequester,
    Modifier, PxSize, RenderSlot, SemanticsModifierNode, State,
    accesskit::{Role, Toggled},
    current_frame_nanos,
    modifier::ModifierCapabilityExt as _,
    receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    checkmark::checkmark,
    modifier::{
        DisabledToggleSemantics, InteractionState, InteractionVisualState, ModifierExt,
        PointerEventContext, ToggleableArgs,
    },
    ripple_state::{RippleSpec, RippleState},
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceStyle, surface},
    theme::{MaterialAlpha, MaterialColorScheme, MaterialTheme},
};

use motion::{Colors, ColorsMotion, Scalar, Spec};

mod motion;

#[cfg(all(test, not(target_family = "wasm")))]
mod accessibility_tests;

#[cfg(all(test, not(target_family = "wasm")))]
mod appearance_tests;

#[cfg(all(test, not(target_family = "wasm")))]
mod motion_tests;

#[cfg(all(test, not(target_family = "wasm")))]
mod state_tests;

struct CheckboxValueSemantics {
    toggled: Toggled,
    is_error: bool,
}

impl SemanticsModifierNode for CheckboxValueSemantics {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.toggled = Some(self.toggled);
        if self.is_error {
            node.invalid = Some(tessera_ui::accesskit::Invalid::True);
        }
    }
}

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
        self.checkmark.checked && !self.checkmark.indeterminate
    }

    /// Returns whether only part of the represented selection is checked.
    pub fn is_indeterminate(&self) -> bool {
        self.checkmark.indeterminate
    }

    /// Sets the checked state directly and resets animation progress.
    pub fn set_checked(&mut self, checked: bool) {
        self.set_value(checked, false);
    }

    /// Sets partial selection without animation. Clearing a partial selection
    /// leaves the checkbox unchecked; an already checked checkbox stays
    /// checked.
    pub fn set_indeterminate(&mut self, indeterminate: bool) {
        self.set_value(self.is_checked(), indeterminate);
    }

    fn set_value(&mut self, checked: bool, indeterminate: bool) {
        let active = checked || indeterminate;
        if self.checkmark.checked != active
            || self.checkmark.indeterminate != indeterminate
            || self.is_animating()
        {
            self.checkmark.snap_to(active, indeterminate);
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
        self.checkmark.progress.is_animating() || self.checkmark.morph.is_animating()
    }
}

#[derive(Clone, PartialEq)]
struct CheckmarkState {
    checked: bool,
    indeterminate: bool,
    progress: Scalar,
    morph: Scalar,
    snap_revision: u64,
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
            indeterminate: false,
            progress: Scalar::new(if initial_state { 1.0 } else { 0.0 }),
            morph: Scalar::new(0.0),
            snap_revision: 0,
        }
    }

    fn snap_to(&mut self, active: bool, indeterminate: bool) {
        self.checked = active;
        self.indeterminate = indeterminate;
        self.progress.snap(if active { 1.0 } else { 0.0 });
        self.morph.snap(if indeterminate { 1.0 } else { 0.0 });
        self.snap_revision = self.snap_revision.wrapping_add(1);
    }

    fn toggle(&mut self) {
        self.animate_to(
            !self.checked || self.indeterminate,
            false,
            current_frame_nanos(),
        );
    }

    fn animate_to(&mut self, checked: bool, indeterminate: bool, frame_nanos: u64) {
        let active = checked || indeterminate;
        if self.checked == active && self.indeterminate == indeterminate {
            return;
        }
        let spec = if active { Spec::Spatial } else { Spec::Removal };
        self.progress
            .retarget(if active { 1.0 } else { 0.0 }, spec, frame_nanos);
        let morph = if indeterminate { 1.0 } else { 0.0 };
        if !self.checked {
            self.morph.snap(morph);
        } else {
            self.morph.retarget(morph, spec, frame_nanos);
        }
        self.checked = active;
        self.indeterminate = indeterminate;
    }

    fn update_progress(&mut self, frame_nanos: u64) {
        self.progress.advance(frame_nanos);
        self.morph.advance(frame_nanos);
    }

    fn progress(&self) -> f32 {
        self.progress.value.clamp(0.0, 1.0) as f32
    }
}

/// # checkbox
///
/// Renders an interactive checkbox with an animated checkmark.
///
/// ## Usage
///
/// Capture a boolean choice or represent partial selection for a group of
/// choices.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the checkbox subtree.
/// - `focus_requester` — optional handle targeting the checkbox for focus
///   requests.
/// - `on_toggle` — toggle callback invoked with the new checked state.
/// - `checked` — externally controlled checked state; omit to use the
///   controller.
/// - `indeterminate` — partial selection, taking precedence over `checked`;
///   activation clears this state and requests `true` through `on_toggle`.
/// - `presentation_state` — optional visual interaction; live input and
///   disabled state take precedence.
/// - `is_error` — validation error state; publishes invalid semantics and uses
///   error colors unless explicitly overridden. Supply validation details with
///   `accessibility_description`. Disabled colors take precedence.
/// - `size` — optional checkbox glyph size.
/// - `color` — optional unchecked outline color.
/// - `checked_color` — optional checked container color.
/// - `checkmark_color` — optional checkmark icon color.
/// - `checkmark_stroke_width` — optional stroke width in physical pixels;
///   defaults to 2dp rounded down to a whole pixel.
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
/// use tessera_components::checkbox::{CheckboxController, checkbox};
/// use tessera_ui::{remember, tessera};
///
/// let mut controller = CheckboxController::new(false);
/// controller.set_indeterminate(true);
/// assert!(controller.is_indeterminate());
/// controller.toggle();
/// assert!(controller.is_checked());
/// assert!(!controller.is_indeterminate());
///
/// #[tessera]
/// fn checkbox_demo() {
///     let choices = remember(|| [true, false]);
///     let all = choices.with(|values| values.iter().all(|value| *value));
///     let some = choices.with(|values| values.iter().any(|value| *value));
///     checkbox()
///         .checked(all)
///         .indeterminate(some && !all)
///         .is_error(!some)
///         .accessibility_description("Choose at least one option")
///         .on_toggle(move |value| choices.set([value; 2]));
/// }
/// ```
#[tessera]
pub fn checkbox(
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    on_toggle: Option<CallbackWith<bool, ()>>,
    checked: Option<bool>,
    indeterminate: Option<bool>,
    is_error: Option<bool>,
    presentation_state: Option<InteractionVisualState>,
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
    controller: Option<State<CheckboxController>>,
) {
    let focus_requester = if let Some(focus_requester) = focus_requester {
        focus_requester
    } else {
        remember(FocusRequester::new).get()
    };
    let modifier = modifier.unwrap_or_default();
    let disabled = disabled.unwrap_or(false);
    let is_error = is_error.unwrap_or(false);
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let size = size.unwrap_or(CheckboxDefaults::GLYPH_SIZE);
    let unchecked_interaction_color = color.unwrap_or(if is_error {
        scheme.error
    } else {
        scheme.on_surface
    });
    let color = color.unwrap_or(if is_error {
        scheme.error
    } else {
        scheme.on_surface_variant
    });
    let checked_color = checked_color.unwrap_or(if is_error {
        scheme.error
    } else {
        scheme.primary
    });
    let checkmark_color = checkmark_color.unwrap_or(if is_error {
        scheme.on_error
    } else {
        scheme.on_primary
    });
    let checkmark_stroke_width =
        checkmark_stroke_width.unwrap_or_else(|| Dp(2.0).to_pixels_f32().floor());
    let shape = shape.unwrap_or_else(CheckboxDefaults::default_shape);
    let disabled_color = disabled_color.unwrap_or(
        scheme
            .on_surface
            .with_alpha(MaterialAlpha::DISABLED_CONTENT),
    );
    let disabled_checkmark_color = disabled_checkmark_color.unwrap_or(scheme.surface);
    let controller = if let Some(controller) = controller {
        controller
    } else {
        remember(|| {
            let mut controller = CheckboxController::new(checked.unwrap_or(false));
            controller.set_indeterminate(indeterminate.unwrap_or(false));
            controller
        })
    };
    if checked.is_some() || indeterminate.is_some() {
        let checked =
            checked.unwrap_or_else(|| controller.with(|controller| controller.is_checked()));
        let indeterminate = indeterminate.unwrap_or(false);
        if controller.with(|controller| {
            controller.checkmark.checked != (checked || indeterminate)
                || controller.is_indeterminate() != indeterminate
        }) {
            controller.with_mut(|controller| {
                controller
                    .checkmark
                    .animate_to(checked, indeterminate, current_frame_nanos())
            });
        }
    }

    checkbox_inner()
        .focus_requester(focus_requester)
        .modifier(modifier)
        .size(size)
        .color(color)
        .unchecked_interaction_color(unchecked_interaction_color)
        .checked_color(checked_color)
        .checkmark_color(checkmark_color)
        .checkmark_stroke_width(checkmark_stroke_width)
        .shape(shape)
        .disabled(disabled)
        .is_error(is_error)
        .presentation_state_optional(presentation_state)
        .disabled_color(disabled_color)
        .disabled_checkmark_color(disabled_checkmark_color)
        .controller(controller)
        .on_toggle_optional(on_toggle)
        .accessibility_label_optional(accessibility_label)
        .accessibility_description_optional(accessibility_description);
}

#[tessera]
fn checkbox_inner(
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    on_toggle: Option<CallbackWith<bool, ()>>,
    size: Option<Dp>,
    color: Option<Color>,
    unchecked_interaction_color: Option<Color>,
    checked_color: Option<Color>,
    checkmark_color: Option<Color>,
    checkmark_stroke_width: Option<f32>,
    shape: Option<Shape>,
    disabled: Option<bool>,
    is_error: Option<bool>,
    presentation_state: Option<InteractionVisualState>,
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
    let checkmark_stroke_width =
        checkmark_stroke_width.unwrap_or_else(|| Dp(2.0).to_pixels_f32().floor());
    let shape = shape.unwrap_or(Shape::CAPSULE);
    let disabled = disabled.unwrap_or(false);
    let is_error = is_error.unwrap_or(false);
    let disabled_color = disabled_color.unwrap_or(Color::TRANSPARENT);
    let disabled_checkmark_color = disabled_checkmark_color.unwrap_or(Color::TRANSPARENT);
    let controller = controller.expect("checkbox_inner requires controller to be set");
    let is_checked = controller.with(|c| c.is_checked());
    let is_indeterminate = controller.with(|controller| controller.is_indeterminate());
    let is_active = is_checked || is_indeterminate;
    let enabled = !disabled;
    let interaction_state = if enabled {
        Some(remember(InteractionState::new))
    } else {
        None
    };
    let ripple_state = if enabled {
        Some(remember(RippleState::new))
    } else {
        None
    };
    let tap_recognizer = if enabled {
        Some(remember(TapRecognizer::default))
    } else {
        None
    };

    let on_value_change = {
        let on_toggle = on_toggle.unwrap_or_else(CallbackWith::default_value);
        CallbackWith::new(move |next_checked| {
            controller.with_mut(|controller| {
                controller
                    .checkmark
                    .animate_to(next_checked, false, current_frame_nanos())
            });
            on_toggle.call(next_checked);
        })
    };

    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let visual = interaction_state
        .map(|state| state.with(|state| state.with_presentation(presentation_state, enabled)))
        .unwrap_or_else(|| InteractionState::new().with_presentation(presentation_state, enabled));
    let color = if visual.is_hovered() || visual.is_focused() || visual.is_pressed() {
        unchecked_interaction_color.unwrap_or(scheme.on_surface)
    } else {
        color
    };
    let border = if disabled {
        disabled_color
    } else if is_active {
        checked_color
    } else {
        color
    };
    let target = Colors {
        fill: if is_active {
            border
        } else {
            Color::TRANSPARENT
        },
        border,
        icon: if disabled {
            disabled_checkmark_color
        } else if is_active {
            checkmark_color
        } else {
            Color::TRANSPARENT
        },
        enabled,
        active: is_active,
        snap_revision: controller.with(|controller| controller.checkmark.snap_revision),
    };
    let frame = current_frame_nanos();
    let colors = remember(|| ColorsMotion::new(target));
    if colors.with(|colors| colors.target != target) {
        colors.with_mut(|colors| colors.retarget(target, frame));
    }
    if controller.with(CheckboxController::is_animating) || colors.with(ColorsMotion::is_animating)
    {
        receive_frame_nanos(move |frame| {
            #[cfg(all(test, not(target_family = "wasm")))]
            motion_tests::TICKS.set(motion_tests::TICKS.get() + 1);
            let stroke_active = controller.with_mut(|controller| {
                controller.update_progress(frame);
                controller.is_animating()
            });
            let color_active = colors.with_mut(|colors| {
                colors.advance(frame);
                colors.is_animating()
            });
            if stroke_active || color_active {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let (fill_color, border_color, icon_color) = colors.with(ColorsMotion::value);
    let outline_width = Dp::from_pixels_f32(Dp(2.0).to_pixels_f32().floor());
    let checkbox_style = if fill_color == border_color {
        SurfaceStyle::Filled { color: fill_color }
    } else if fill_color.a == 0.0 {
        SurfaceStyle::Outlined {
            color: border_color,
            width: outline_width,
        }
    } else {
        SurfaceStyle::FilledOutlined {
            fill_color,
            border_color,
            border_width: outline_width,
        }
    };

    let state_layer_base = if is_error {
        if is_active { checked_color } else { color }
    } else {
        CheckboxDefaults::state_layer_base_color(is_active, checked_color, &scheme)
    };

    let checkmark_slot = RenderSlot::new(move || {
        let progress = controller.with(|c| c.progress());
        let morph = controller.with(|controller| controller.checkmark.morph.value as f32);
        if progress > 0.0 {
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    checkmark()
                        .color(icon_color)
                        .stroke_width(checkmark_stroke_width)
                        .progress(progress)
                        .indeterminate_fraction(morph)
                        .size(size)
                        .padding([0.0, 0.0])
                        .checkbox_geometry(true);
                });
        }
    });

    let checkbox_surface_slot = {
        RenderSlot::new(move || {
            let checkbox_style = checkbox_style.clone();
            surface()
                .modifier(Modifier::new().size(size, size))
                .shape(shape)
                .style(checkbox_style)
                .child(move || {
                    checkmark_slot.render();
                });
        })
    };

    let checkbox_container_slot = {
        RenderSlot::new(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    checkbox_surface_slot.render();
                });
        })
    };

    let state_layer_slot = {
        RenderSlot::new(move || {
            surface()
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
                .interaction_state_optional(interaction_state)
                .presentation_state_optional(presentation_state)
                .focus_requester_optional(focus_requester)
                .show_focus_indicator(true)
                .ripple_state_optional(ripple_state);
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
            focus_requester,
            value: is_checked,
            on_value_change,
            enabled: true,
            role: Some(Role::CheckBox),
            label: accessibility_label.clone(),
            description: accessibility_description.clone(),
            interaction_state,
            on_press: press_handler.map(Into::into),
            on_release: release_handler.map(Into::into),
            tap_recognizer,
            ..Default::default()
        };
        modifier = modifier.toggleable_with(toggle_args);
    } else {
        modifier = modifier.push_semantics(DisabledToggleSemantics {
            role: Role::CheckBox,
            value: is_checked,
            label: accessibility_label,
            description: accessibility_description,
        });
    }
    let modifier = modifier.push_semantics(CheckboxValueSemantics {
        toggled: if is_indeterminate {
            Toggled::Mixed
        } else if is_checked {
            Toggled::True
        } else {
            Toggled::False
        },
        is_error,
    });
    boxed()
        .modifier(modifier)
        .alignment(Alignment::Center)
        .children(move || {
            state_layer_slot.render();
            checkbox_container_slot.render();
        });
}

#[cfg(test)]
mod tests {
    use super::CheckboxController;

    #[test]
    fn checkbox_controller_animates_to_checked_state() {
        let mut controller = CheckboxController::new(false);

        controller.checkmark.animate_to(true, false, 1_000_000_000);

        assert!(controller.is_checked());
        assert!(controller.is_animating());
        assert_eq!(controller.progress(), 0.0);

        controller.update_progress(1_100_000_000);
        assert!((controller.progress() - 0.65983105).abs() < 0.00001);
        assert!(controller.is_animating());

        controller.set_checked(true);
        assert_eq!(controller.progress(), 1.0);
        assert!(!controller.is_animating());
    }

    #[test]
    fn checkbox_controller_animates_to_unchecked_state() {
        let mut controller = CheckboxController::new(true);

        controller.checkmark.animate_to(false, false, 1_000_000_000);

        assert!(!controller.is_checked());
        assert!(controller.is_animating());
        assert_eq!(controller.progress(), 1.0);

        controller.update_progress(1_099_000_000);
        assert_eq!(controller.progress(), 1.0);
        assert!(controller.is_animating());

        controller.update_progress(1_100_000_000);
        assert_eq!(controller.progress(), 0.0);
        assert!(!controller.is_animating());
    }

    #[test]
    fn mixed_activation_selects_and_morph_retargeting_preserves_the_stroke() {
        let mut controller = CheckboxController::new(false);
        controller.set_indeterminate(true);
        assert!(controller.is_indeterminate());
        assert!(!controller.is_checked());
        let start = tessera_ui::current_frame_nanos();
        controller.toggle();
        assert!(controller.is_checked());
        assert!(!controller.is_indeterminate());
        controller.update_progress(start + 100_000_000);
        assert_eq!(controller.progress(), 1.0);
        let morph = controller.checkmark.morph.value;
        let velocity = controller.checkmark.morph.velocity;
        assert!((morph - 0.34016894).abs() < 0.00001);
        controller
            .checkmark
            .animate_to(false, true, start + 100_000_000);
        controller.update_progress(start + 100_000_000);
        assert!((controller.checkmark.morph.value - morph).abs() < 0.000000000001);
        assert!((controller.checkmark.morph.velocity - velocity).abs() < 0.000000000001);
        controller.update_progress(start + 120_000_000);
        assert!(
            controller.checkmark.morph.value < morph,
            "reversal retains incoming velocity"
        );
        controller.update_progress(start + 600_000_000);
        assert_eq!(controller.checkmark.morph.value, 1.0);
        assert!(controller.is_indeterminate());
        assert!(!controller.is_animating());
        controller
            .checkmark
            .animate_to(true, false, start + 600_000_000);
        controller.update_progress(start + 700_000_000);
        let held_morph = controller.checkmark.morph.value;
        controller
            .checkmark
            .animate_to(false, false, start + 700_000_000);
        controller.update_progress(start + 799_000_000);
        assert!((controller.checkmark.morph.value - held_morph).abs() < 0.000000000001);
        controller.update_progress(start + 800_000_000);
        assert_eq!(controller.checkmark.morph.value, 0.0);
        assert_eq!(controller.progress(), 0.0);
        assert!(!controller.is_animating());
    }
}
