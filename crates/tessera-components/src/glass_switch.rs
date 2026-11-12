//! A switch (toggle) component with a glassmorphic visual style.
//!
//! ## Usage
//!
//! Use in settings, forms, or toolbars to control a boolean state.
use std::time::Duration;

use tessera_foundation::modifier::{InteractionState, ModifierExt as _, ToggleableArgs};
use tessera_ui::{
    AxisConstraint, CallbackWith, Color, ComputedData, Constraint, Dp, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition,
    accesskit::Role,
    launch_once,
    layout::{MeasureScope, PlacementResult, PlacementScope},
    remember, tessera,
    time::Instant,
};

use crate::{
    animation::{self, run_frames},
    fluid_glass::{GlassBorder, fluid_glass},
    modifier::ModifierExt as _,
    shape_def::Shape,
};

const ANIMATION_DURATION: Duration = Duration::from_millis(150);

#[derive(Clone, PartialEq)]
struct GlassSwitchMotion {
    checked: bool,
    progress: f32,
    animating: bool,
    // The first frame that observes a toggle starts its animation.
    animation_start: Option<Instant>,
}

impl GlassSwitchMotion {
    fn new(initial_state: bool) -> Self {
        Self {
            checked: initial_state,
            progress: if initial_state { 1.0 } else { 0.0 },
            animating: false,
            animation_start: None,
        }
    }

    fn is_checked(&self) -> bool {
        self.checked
    }

    fn toggle(&mut self) {
        self.checked = !self.checked;
        self.animating = true;
        self.animation_start = None;
    }

    fn update_progress(&mut self, now: Instant) -> bool {
        if self.animating {
            let start = *self.animation_start.get_or_insert(now);
            let fraction = (now.saturating_duration_since(start).as_secs_f32()
                / ANIMATION_DURATION.as_secs_f32())
            .min(1.0);
            let target = if self.checked { 1.0 } else { 0.0 };
            self.progress = target * fraction + (1.0 - fraction) * (1.0 - target);
            if fraction >= 1.0 {
                self.animating = false;
                self.animation_start = None;
                self.progress = target;
            }
        }
        self.animating
    }

    fn animation_progress(&self) -> f32 {
        self.progress
    }

    fn is_animating(&self) -> bool {
        self.animating
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
/// - `checked` — whether the switch is on.
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
///
/// ## Examples
///
/// ```
/// use tessera_components::glass_switch::glass_switch;
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{LayoutResult, remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let checked = remember(|| false);
///     assert!(!checked.get());
///
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(move || {
///             glass_switch(checked.get()).on_toggle(move |value| checked.set(value));
///         });
///
/// }
///
/// # tessera_ui::testing::UiTest::new().content(demo).frame();
/// ```
#[tessera]
pub fn glass_switch(
    checked: bool,
    modifier: Option<Modifier>,
    on_toggle: Option<CallbackWith<bool, ()>>,
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
) {
    let width = width.unwrap_or(Dp(52.0));
    let height = height.unwrap_or(Dp(32.0));
    let track_on_color = track_on_color.unwrap_or(Color::new(0.2, 0.7, 1.0, 0.5));
    let track_off_color = track_off_color.unwrap_or(Color::new(0.8, 0.8, 0.8, 0.5));
    let thumb_on_alpha = thumb_on_alpha.unwrap_or(0.5);
    let thumb_off_alpha = thumb_off_alpha.unwrap_or(1.0);
    let thumb_padding = thumb_padding.unwrap_or(Dp(3.0));
    let controller = remember(|| GlassSwitchMotion::new(checked));
    if controller.with(|c| c.is_checked()) != checked {
        controller.with_mut(|c| c.toggle());
    }
    let mut modifier = modifier.unwrap_or_default();

    let enabled = on_toggle.is_some();
    let interaction_state = if enabled {
        Some(remember(InteractionState::new))
    } else {
        None
    };
    if enabled {
        modifier = modifier.minimum_interactive_component_size();
        let toggle_args = ToggleableArgs {
            value: checked,
            on_value_change: CallbackWith::new(move |checked| {
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
        modifier = modifier.toggleable(toggle_args);
    }

    let width_px = width.to_px();
    let height_px = height.to_px();
    let thumb_dp = Dp(height.0 - (thumb_padding.0 * 2.0));
    let thumb_px = thumb_dp.to_px();

    if controller.with(|c| c.is_animating()) {
        launch_once(run_frames(controller, GlassSwitchMotion::update_progress));
    }
    let progress = controller.with(|c| c.animation_progress());
    let track_color = interpolate_color(track_off_color, track_on_color, progress);

    tessera_ui::layout::layout(GlassSwitchLayout {
        width: width_px,
        height: height_px,
        thumb_padding: thumb_padding.to_px(),
        progress,
    })
    .modifier(modifier)
    .child(move || {
        fluid_glass()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(width_px)),
                Some(AxisConstraint::exact(height_px)),
            ))
            .tint_color(track_color)
            .shape(Shape::CAPSULE)
            .blur_radius(Dp(8.0))
            .border_optional(track_border)
            .child(|| {});

        let thumb_alpha = thumb_off_alpha + (thumb_on_alpha - thumb_off_alpha) * progress;
        let thumb_color = Color::new(1.0, 1.0, 1.0, thumb_alpha);
        fluid_glass()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(thumb_px)),
                Some(AxisConstraint::exact(thumb_px)),
            ))
            .tint_color(thumb_color)
            .shape(Shape::Ellipse)
            .border_optional(thumb_border)
            .child(|| {});
    });
}

#[derive(Clone, PartialEq)]
struct GlassSwitchLayout {
    width: Px,
    height: Px,
    thumb_padding: Px,
    progress: f32,
}

impl tessera_ui::LayoutPolicy for GlassSwitchLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let track = children[0];
        let thumb = children[1];

        let track_constraint = Constraint::exact(self.width, self.height);
        let thumb_constraint = Constraint::NONE;

        let _ = track.measure(input, &track_constraint)?;
        let thumb_size = thumb.measure(input, &thumb_constraint)?;

        let eased_progress = animation::easing(self.progress);

        result.place_child(track, PxPosition::new(Px(0), Px(0)));

        let start_x = self.thumb_padding;
        let end_x = self.width - thumb_size.width - self.thumb_padding;
        let thumb_x = start_x.0 as f32 + (end_x.0 - start_x.0) as f32 * eased_progress;
        let thumb_y = (self.height - thumb_size.height) / 2;

        result.place_child(thumb, PxPosition::new(Px(thumb_x as i32), thumb_y));

        Ok(result.with_size(ComputedData {
            width: self.width,
            height: self.height,
        }))
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

    fn place(
        &self,
        input: &PlacementScope<'_>,
        _inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        let mut result = LayoutResult::default();
        let children = input.children();
        if children.len() < 2 {
            return Some(result.into_placement_result());
        }
        let track = children[0];
        let thumb = children[1];
        let thumb_size = thumb.size();

        let eased_progress = animation::easing(self.progress);
        result.place_child(track, PxPosition::new(Px(0), Px(0)));

        let start_x = self.thumb_padding;
        let end_x = self.width - thumb_size.width - self.thumb_padding;
        let thumb_x = start_x.0 as f32 + (end_x.0 - start_x.0) as f32 * eased_progress;
        let thumb_y = (self.height - thumb_size.height) / 2;

        result.place_child(thumb, PxPosition::new(Px(thumb_x as i32), thumb_y));
        Some(result.into_placement_result())
    }
}
