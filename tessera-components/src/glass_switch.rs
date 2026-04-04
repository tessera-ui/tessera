//! A switch (toggle) component with a glassmorphic visual style.
//!
//! ## Usage
//!
//! Use in settings, forms, or toolbars to control a boolean state.
use std::time::Duration;

use tessera_ui::{
    AxisConstraint, CallbackWith, Color, ComputedData, Constraint, Dp, MeasurementError, Modifier,
    Px, PxPosition, State,
    accesskit::Role,
    current_frame_nanos,
    layout::{LayoutInput, LayoutOutput, LayoutPolicy, PlacementInput, layout_primitive},
    receive_frame_nanos, remember, tessera,
};

use crate::{
    animation,
    fluid_glass::{GlassBorder, fluid_glass},
    modifier::{InteractionState, ModifierExt as _, ToggleableArgs},
    shape_def::Shape,
};

const ANIMATION_DURATION: Duration = Duration::from_millis(150);

/// Controller for the `glass_switch` component.
#[derive(Clone, PartialEq)]
pub struct GlassSwitchController {
    checked: bool,
    progress: f32,
    last_toggle_frame_nanos: Option<u64>,
}

impl GlassSwitchController {
    /// Creates a new controller with the given initial value.
    pub fn new(initial_state: bool) -> Self {
        Self {
            checked: initial_state,
            progress: if initial_state { 1.0 } else { 0.0 },
            last_toggle_frame_nanos: None,
        }
    }

    /// Returns whether the switch is currently checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Sets the checked state directly, resetting animation progress.
    pub fn set_checked(&mut self, checked: bool) {
        if self.checked != checked {
            self.checked = checked;
            self.progress = if checked { 1.0 } else { 0.0 };
            self.last_toggle_frame_nanos = None;
        }
    }

    /// Toggles the switch and starts the animation timeline.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
        self.last_toggle_frame_nanos = Some(current_frame_nanos());
    }

    /// Advances the animation timeline based on elapsed time.
    pub fn update_progress(&mut self, frame_nanos: u64) {
        if let Some(start_frame_nanos) = self.last_toggle_frame_nanos {
            let elapsed_nanos = frame_nanos.saturating_sub(start_frame_nanos);
            let animation_nanos = ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
            let fraction = if animation_nanos == 0 {
                1.0
            } else {
                (elapsed_nanos as f32 / animation_nanos as f32).min(1.0)
            };
            let target = if self.checked { 1.0 } else { 0.0 };
            self.progress = target * fraction + (1.0 - fraction) * (1.0 - target);
            if fraction >= 1.0 {
                self.last_toggle_frame_nanos = None;
                self.progress = target;
            }
        }
    }

    /// Returns the current animation progress (0.0..1.0).
    pub fn animation_progress(&self) -> f32 {
        self.progress
    }

    /// Returns whether the toggle animation is currently running.
    pub fn is_animating(&self) -> bool {
        self.last_toggle_frame_nanos.is_some()
    }
}

impl Default for GlassSwitchController {
    fn default() -> Self {
        Self::new(false)
    }
}

fn interpolate_color(off: Color, on: Color, progress: f32) -> Color {
    Color {
        r: off.r + (on.r - off.r) * progress,
        g: off.g + (on.g - off.g) * progress,
        b: off.b + (on.b - off.b) * progress,
        a: off.a + (on.a - off.a) * progress,
    }
}

/// # glass_switch
///
/// An interactive switch with glass effect.
///
/// ## Usage
///
/// Use when you need a toggle switch with a glassmorphic style.
///
/// ## Parameters
///
/// - `modifier` — optional modifier chain applied to the switch subtree.
/// - `on_toggle` — optional callback invoked when the switch toggles.
/// - `checked` — initial checked state.
/// - `width` — optional track width override.
/// - `height` — optional track height override.
/// - `track_on_color` — optional track tint when checked.
/// - `track_off_color` — optional track tint when unchecked.
/// - `thumb_on_alpha` — optional thumb alpha when checked.
/// - `thumb_off_alpha` — optional thumb alpha when unchecked.
/// - `thumb_border` — optional thumb border override.
/// - `track_border` — optional track border override.
/// - `thumb_padding` — optional thumb padding.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `controller` — optional external controller.
///
/// ## Examples
///
/// ```
/// use tessera_components::glass_switch::{GlassSwitchController, glass_switch};
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let controller = remember(|| GlassSwitchController::new(false));
///     assert!(!controller.with(|c| c.is_checked()));
///
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(move || {
///             glass_switch().checked(controller.with(|c| c.is_checked()));
///         });
///
///     controller.with_mut(|c| c.toggle());
///     assert!(controller.with(|c| c.is_checked()));
/// }
///
/// demo();
/// ```
#[tessera]
pub fn glass_switch(
    modifier: Option<Modifier>,
    on_toggle: Option<CallbackWith<bool, ()>>,
    checked: bool,
    width: Option<Dp>,
    height: Option<Dp>,
    track_on_color: Option<Color>,
    track_off_color: Option<Color>,
    thumb_on_alpha: Option<f32>,
    thumb_off_alpha: Option<f32>,
    thumb_border: Option<GlassBorder>,
    track_border: Option<GlassBorder>,
    thumb_padding: Option<Dp>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    #[prop(skip_setter)] controller: Option<State<GlassSwitchController>>,
) {
    let width = width.unwrap_or(Dp(52.0));
    let height = height.unwrap_or(Dp(32.0));
    let track_on_color = track_on_color.unwrap_or(Color::new(0.2, 0.7, 1.0, 0.5));
    let track_off_color = track_off_color.unwrap_or(Color::new(0.8, 0.8, 0.8, 0.5));
    let thumb_on_alpha = thumb_on_alpha.unwrap_or(0.5);
    let thumb_off_alpha = thumb_off_alpha.unwrap_or(1.0);
    let thumb_padding = thumb_padding.unwrap_or(Dp(3.0));
    let controller = controller.unwrap_or_else(|| remember(|| GlassSwitchController::new(checked)));
    if controller.with(|c| c.is_checked()) != checked {
        controller.with_mut(|c| c.set_checked(checked));
    }
    let mut modifier = modifier.unwrap_or_default();

    let enabled = on_toggle.is_some();
    let interaction_state = enabled.then(|| remember(InteractionState::new));
    let checked = controller.with(|c| c.is_checked());
    if enabled {
        modifier = modifier.minimum_interactive_component_size();
        let toggle_args = ToggleableArgs {
            value: checked,
            on_value_change: CallbackWith::new(move |_| {
                controller.with_mut(|c| c.toggle());
                let checked = controller.with(|c| c.is_checked());
                if let Some(on_toggle) = on_toggle.as_ref() {
                    on_toggle.call(checked);
                }
            }),
            enabled: true,
            role: Some(Role::Switch),
            label: accessibility_label.clone(),
            description: accessibility_description.clone(),
            interaction_state,
            ..Default::default()
        };
        modifier = modifier.toggleable_with(toggle_args);
    }

    // Precompute pixel sizes to avoid repeated conversions
    let width_px = width.to_px();
    let height_px = height.to_px();
    let thumb_dp = Dp(height.0 - (thumb_padding.0 * 2.0));
    let thumb_px = thumb_dp.to_px();

    // Track tint color interpolation based on progress
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
    let progress = controller.with(|c| c.animation_progress());
    let track_color = interpolate_color(track_off_color, track_on_color, progress);

    layout_primitive()
        .modifier(modifier)
        .layout_policy(GlassSwitchLayout {
            width: width_px,
            height: height_px,
            thumb_padding: thumb_padding.to_px(),
            progress,
        })
        .child(move || {
            let track = fluid_glass()
                .modifier(Modifier::new().constrain(
                    Some(AxisConstraint::exact(width_px)),
                    Some(AxisConstraint::exact(height_px)),
                ))
                .tint_color(track_color)
                .shape(Shape::capsule())
                .blur_radius(Dp(8.0));
            if let Some(border) = track_border {
                track.border(border).with_child(|| {});
            } else {
                track.with_child(|| {});
            }

            let thumb_alpha = thumb_off_alpha + (thumb_on_alpha - thumb_off_alpha) * progress;
            let thumb_color = Color::new(1.0, 1.0, 1.0, thumb_alpha);
            let thumb = fluid_glass()
                .modifier(Modifier::new().constrain(
                    Some(AxisConstraint::exact(thumb_px)),
                    Some(AxisConstraint::exact(thumb_px)),
                ))
                .tint_color(thumb_color)
                .refraction_height(Dp(1.0))
                .shape(Shape::Ellipse);
            if let Some(border) = thumb_border {
                thumb.border(border).with_child(|| {});
            } else {
                thumb.with_child(|| {});
            }
        });
}

#[derive(Clone, PartialEq)]
struct GlassSwitchLayout {
    width: Px,
    height: Px,
    thumb_padding: Px,
    progress: f32,
}

impl LayoutPolicy for GlassSwitchLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let track_id = input.children_ids()[0];
        let thumb_id = input.children_ids()[1];

        let track_constraint = Constraint::exact(self.width, self.height);
        let thumb_constraint = Constraint::NONE;

        let nodes_constraints = vec![(track_id, track_constraint), (thumb_id, thumb_constraint)];
        let sizes_map = input.measure_children(nodes_constraints)?;

        let thumb_size = sizes_map
            .get(&thumb_id)
            .expect("thumb size should be measured");

        let eased_progress = animation::easing(self.progress);

        output.place_child(track_id, PxPosition::new(Px(0), Px(0)));

        let start_x = self.thumb_padding;
        let end_x = self.width - thumb_size.width - self.thumb_padding;
        let thumb_x = start_x.0 as f32 + (end_x.0 - start_x.0) as f32 * eased_progress;
        let thumb_y = (self.height - thumb_size.height) / 2;

        output.place_child(thumb_id, PxPosition::new(Px(thumb_x as i32), thumb_y));

        Ok(ComputedData {
            width: self.width,
            height: self.height,
        })
    }

    fn measure_eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.thumb_padding == other.thumb_padding
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.thumb_padding == other.thumb_padding
            && self.progress == other.progress
    }

    fn place_children(&self, input: &PlacementInput<'_>, output: &mut LayoutOutput<'_>) -> bool {
        let ids = input.children_ids();
        if ids.len() < 2 {
            return true;
        }
        let track_id = ids[0];
        let thumb_id = ids[1];
        let Some(thumb_size) = input.child_size(thumb_id) else {
            return false;
        };

        let eased_progress = animation::easing(self.progress);
        output.place_child(track_id, PxPosition::new(Px(0), Px(0)));

        let start_x = self.thumb_padding;
        let end_x = self.width - thumb_size.width - self.thumb_padding;
        let thumb_x = start_x.0 as f32 + (end_x.0 - start_x.0) as f32 * eased_progress;
        let thumb_y = (self.height - thumb_size.height) / 2;

        output.place_child(thumb_id, PxPosition::new(Px(thumb_x as i32), thumb_y));
        true
    }
}
