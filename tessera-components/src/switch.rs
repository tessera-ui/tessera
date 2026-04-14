//! An interactive toggle switch component.
//!
//! ## Usage
//!
//! Use to control a boolean on/off state.
use std::time::Duration;

use tessera_ui::{
    AxisConstraint, CallbackWith, Color, ComputedData, Constraint, Dp, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, PxSize, RenderSlot, State,
    accesskit::Role,
    current_frame_nanos,
    layout::{LayoutPolicy, MeasureScope, PlacementScope, layout},
    receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    animation,
    boxed::boxed,
    modifier::{InteractionState, ModifierExt, PointerEventContext, ToggleableArgs},
    ripple_state::{RippleSpec, RippleState},
    shape_def::Shape,
    surface::{SurfaceStyle, surface},
    theme::{ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme},
};

const ANIMATION_DURATION: Duration = Duration::from_millis(150);

#[cfg(test)]
const THUMB_TEST_TAG: &str = "__switch_thumb";

/// Material Design 3 defaults for [`switch`].
pub struct SwitchDefaults;

struct SwitchColorInputs {
    track_color: Color,
    track_checked_color: Color,
    track_outline_color: Color,
    thumb_color: Color,
    thumb_checked_color: Color,
    thumb_checked_icon_color: Color,
    thumb_icon_color: Color,
}

struct SwitchStateFlags {
    checked: bool,
    enabled: bool,
}

impl SwitchDefaults {
    /// Default track width.
    pub const WIDTH: Dp = Dp(52.0);
    /// Default track height.
    pub const HEIGHT: Dp = Dp(32.0);
    /// Default state layer size (unbounded ripple/hover target around the
    /// thumb).
    pub const STATE_LAYER_SIZE: Dp = Dp(40.0);
    /// Thumb diameter when checked or when it contains content.
    pub const THUMB_DIAMETER: Dp = Dp(24.0);
    /// Thumb diameter when unchecked and it has no content.
    pub const UNCHECKED_THUMB_DIAMETER: Dp = Dp(16.0);
    /// Thumb diameter when pressed.
    pub const PRESSED_THUMB_DIAMETER: Dp = Dp(28.0);
    /// Default track outline width.
    pub const TRACK_OUTLINE_WIDTH: Dp = Dp(2.0);

    /// Resolves effective colors for the current state.
    fn resolve_colors(
        inputs: SwitchColorInputs,
        scheme: &MaterialColorScheme,
        flags: SwitchStateFlags,
    ) -> SwitchResolvedColors {
        let mut track_color = if flags.checked {
            inputs.track_checked_color
        } else {
            inputs.track_color
        };
        let mut track_outline_color = if flags.checked {
            Color::TRANSPARENT
        } else {
            inputs.track_outline_color
        };
        let mut thumb_color = if flags.checked {
            inputs.thumb_checked_color
        } else {
            inputs.thumb_color
        };
        let mut icon_color = if flags.checked {
            inputs.thumb_checked_icon_color
        } else {
            inputs.thumb_icon_color
        };

        if !flags.enabled {
            let disabled_container_alpha = MaterialAlpha::DISABLED_CONTAINER;
            let disabled_content_alpha = MaterialAlpha::DISABLED_CONTENT;

            if flags.checked {
                thumb_color = scheme.surface;
                icon_color = scheme
                    .surface
                    .blend_over(scheme.on_surface, disabled_content_alpha);
                track_color = scheme
                    .surface
                    .blend_over(scheme.on_surface, disabled_container_alpha);
                track_outline_color = Color::TRANSPARENT;
            } else {
                thumb_color = scheme
                    .surface
                    .blend_over(scheme.on_surface, disabled_content_alpha);
                icon_color = scheme
                    .surface
                    .blend_over(scheme.surface_container_highest, disabled_content_alpha);
                track_color = scheme
                    .surface
                    .blend_over(scheme.surface_container_highest, disabled_container_alpha);
                track_outline_color = scheme
                    .surface
                    .blend_over(scheme.on_surface, disabled_container_alpha);
            }
        }

        SwitchResolvedColors {
            track_color,
            track_outline_color,
            thumb_color,
            icon_color,
        }
    }
}

struct SwitchResolvedColors {
    track_color: Color,
    track_outline_color: Color,
    thumb_color: Color,
    icon_color: Color,
}

#[derive(Clone)]
struct SwitchLayout {
    track_width: Dp,
    track_height: Dp,
    track_outline_width: Dp,
    thumb_diameter: Dp,
    progress: f64,
    checked: bool,
    is_pressed: bool,
}

impl PartialEq for SwitchLayout {
    fn eq(&self, other: &Self) -> bool {
        self.track_width == other.track_width
            && self.track_height == other.track_height
            && self.track_outline_width == other.track_outline_width
            && self.thumb_diameter == other.thumb_diameter
            && self.progress == other.progress
            && self.checked == other.checked
            && self.is_pressed == other.is_pressed
    }
}

impl LayoutPolicy for SwitchLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let track = children[0];
        let state_layer = children[1];
        let thumb = children[2];
        let thumb_constraint = Constraint::NONE;
        let track_size = track.measure(&thumb_constraint)?;
        let state_layer_size = state_layer.measure(&thumb_constraint)?;
        let thumb_size = thumb.measure(&thumb_constraint)?;

        let self_width_px = track_size
            .width
            .max(state_layer_size.width)
            .max(thumb_size.width);
        let self_height_px = track_size
            .height
            .max(state_layer_size.height)
            .max(thumb_size.height);
        let track_origin_x = (self_width_px.0 - track_size.width.0) / 2;
        let track_origin_y = (self_height_px.0 - track_size.height.0) / 2;

        let checked_thumb_diameter = SwitchDefaults::THUMB_DIAMETER;
        let thumb_padding_start = Dp((self.track_height.0 - checked_thumb_diameter.0) / 2.0);
        let max_bound_dp =
            Dp((self.track_width.0 - checked_thumb_diameter.0) - thumb_padding_start.0);

        let min_bound_dp = Dp((self.track_height.0 - self.thumb_diameter.0) / 2.0);
        let anim_offset_dp = Dp(min_bound_dp.0 + (max_bound_dp.0 - min_bound_dp.0) * self.progress);
        let offset_dp = if self.is_pressed && self.checked {
            Dp(max_bound_dp.0 - self.track_outline_width.0)
        } else if self.is_pressed && !self.checked {
            self.track_outline_width
        } else {
            anim_offset_dp
        };
        let thumb_x = offset_dp.to_px();

        let thumb_center_x = track_origin_x + thumb_x.0 + thumb_size.width.0 / 2;
        let thumb_center_y = track_origin_y + track_size.height.0 / 2;
        let state_layer_x = thumb_center_x - state_layer_size.width.0 / 2;
        let state_layer_y = thumb_center_y - state_layer_size.height.0 / 2;

        result.place_child(
            track,
            PxPosition::new(Px(track_origin_x), Px(track_origin_y)),
        );
        result.place_child(
            thumb,
            PxPosition::new(
                Px(track_origin_x + thumb_x.0),
                Px(track_origin_y + (track_size.height.0 - thumb_size.height.0) / 2),
            ),
        );
        result.place_child(
            state_layer,
            PxPosition::new(Px(state_layer_x), Px(state_layer_y)),
        );

        Ok(result.with_size(ComputedData {
            width: self_width_px,
            height: self_height_px,
        }))
    }

    fn measure_eq(&self, other: &Self) -> bool {
        self.track_width == other.track_width
            && self.track_height == other.track_height
            && self.track_outline_width == other.track_outline_width
            && self.thumb_diameter == other.thumb_diameter
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.track_width == other.track_width
            && self.track_height == other.track_height
            && self.track_outline_width == other.track_outline_width
            && self.thumb_diameter == other.thumb_diameter
            && self.progress == other.progress
            && self.checked == other.checked
            && self.is_pressed == other.is_pressed
    }

    fn place_children(&self, input: &PlacementScope<'_>) -> Option<Vec<(u64, PxPosition)>> {
        let mut result = LayoutResult::default();
        let children = input.children();
        if children.len() < 3 {
            return Some(result.into_placements());
        }
        let track = children[0];
        let state_layer = children[1];
        let thumb = children[2];
        let track_size = track.size();
        let state_layer_size = state_layer.size();
        let thumb_size = thumb.size();

        let self_width_px = input.size().width;
        let self_height_px = input.size().height;
        let track_origin_x = (self_width_px.0 - track_size.width.0) / 2;
        let track_origin_y = (self_height_px.0 - track_size.height.0) / 2;

        let checked_thumb_diameter = SwitchDefaults::THUMB_DIAMETER;
        let thumb_padding_start = Dp((self.track_height.0 - checked_thumb_diameter.0) / 2.0);
        let max_bound_dp =
            Dp((self.track_width.0 - checked_thumb_diameter.0) - thumb_padding_start.0);

        let min_bound_dp = Dp((self.track_height.0 - self.thumb_diameter.0) / 2.0);
        let anim_offset_dp = Dp(min_bound_dp.0 + (max_bound_dp.0 - min_bound_dp.0) * self.progress);
        let offset_dp = if self.is_pressed && self.checked {
            Dp(max_bound_dp.0 - self.track_outline_width.0)
        } else if self.is_pressed && !self.checked {
            self.track_outline_width
        } else {
            anim_offset_dp
        };
        let thumb_x = offset_dp.to_px();

        let thumb_center_x = track_origin_x + thumb_x.0 + thumb_size.width.0 / 2;
        let thumb_center_y = track_origin_y + track_size.height.0 / 2;
        let state_layer_x = thumb_center_x - state_layer_size.width.0 / 2;
        let state_layer_y = thumb_center_y - state_layer_size.height.0 / 2;

        result.place_child(
            track,
            PxPosition::new(Px(track_origin_x), Px(track_origin_y)),
        );
        result.place_child(
            thumb,
            PxPosition::new(
                Px(track_origin_x + thumb_x.0),
                Px(track_origin_y + (track_size.height.0 - thumb_size.height.0) / 2),
            ),
        );
        result.place_child(
            state_layer,
            PxPosition::new(Px(state_layer_x), Px(state_layer_y)),
        );
        Some(result.into_placements())
    }
}

#[derive(Clone, PartialEq)]
struct SwitchThumbLayout {
    size: Px,
}

impl LayoutPolicy for SwitchThumbLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let constraint = Constraint::new(
            AxisConstraint::exact(self.size),
            AxisConstraint::exact(self.size),
        );

        for child in input.children() {
            let _ = child.measure(&constraint)?;
            result.place_child(child, PxPosition::ZERO);
        }

        Ok(result.with_size(ComputedData {
            width: self.size,
            height: self.size,
        }))
    }
}

/// Controller for the `switch` component.
pub struct SwitchController {
    checked: bool,
    progress: f32,
    last_toggle_frame_nanos: Option<u64>,
}

impl SwitchController {
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

    /// Toggles the switch and kicks off the animation timeline.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
        self.last_toggle_frame_nanos = Some(current_frame_nanos());
    }

    /// Returns the current animation progress (0.0..1.0).
    pub fn animation_progress(&self) -> f32 {
        self.progress
    }

    /// Returns whether the switch animation is currently running.
    pub fn is_animating(&self) -> bool {
        self.last_toggle_frame_nanos.is_some()
    }

    /// Advances the animation timeline based on elapsed time.
    fn update_progress(&mut self, frame_nanos: u64) {
        if let Some(last_toggle_frame_nanos) = self.last_toggle_frame_nanos {
            let elapsed_nanos = frame_nanos.saturating_sub(last_toggle_frame_nanos);
            let animation_nanos = ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
            let fraction = if animation_nanos == 0 {
                1.0
            } else {
                (elapsed_nanos as f32 / animation_nanos as f32).min(1.0)
            };
            let target = if self.checked { 1.0 } else { 0.0 };
            let progress = if self.checked {
                fraction
            } else {
                1.0 - fraction
            };

            self.progress = progress;

            if (progress - target).abs() < f32::EPSILON || fraction >= 1.0 {
                self.progress = target;
                self.last_toggle_frame_nanos = None;
            }
        }
    }
}

impl Default for SwitchController {
    fn default() -> Self {
        Self::new(false)
    }
}

/// # switch
///
/// Render a Material switch for boolean on/off input in settings or forms.
///
/// ## Usage
///
/// Use when you want a standard on/off switch without custom thumb content.
///
/// ## Parameters
///
/// - `modifier` — optional modifier chain applied to the switch subtree.
/// - `on_toggle` — optional callback invoked when the switch toggles.
/// - `enabled` — optional enabled state; defaults to `true`.
/// - `checked` — initial checked state.
/// - `width` — optional track width override.
/// - `height` — optional track height override.
/// - `track_color` — optional track color when unchecked.
/// - `track_checked_color` — optional track color when checked.
/// - `track_outline_color` — optional track outline color when unchecked.
/// - `track_outline_width` — optional outline width.
/// - `thumb_color` — optional thumb color when unchecked.
/// - `thumb_checked_color` — optional thumb color when checked.
/// - `thumb_checked_icon_color` — optional icon color when checked.
/// - `thumb_icon_color` — optional icon color when unchecked.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `controller` — optional external controller.
/// - `child` — optional content rendered at the thumb center.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::switch::switch;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// material_theme()
///     .theme(|| MaterialTheme::default())
///     .child(|| {
///         switch().on_toggle(|checked| {
///             assert!(checked || !checked);
///         });
///     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn switch(
    modifier: Option<Modifier>,
    on_toggle: Option<CallbackWith<bool, ()>>,
    enabled: Option<bool>,
    checked: Option<bool>,
    width: Option<Dp>,
    height: Option<Dp>,
    track_color: Option<Color>,
    track_checked_color: Option<Color>,
    track_outline_color: Option<Color>,
    track_outline_width: Option<Dp>,
    thumb_color: Option<Color>,
    thumb_checked_color: Option<Color>,
    thumb_checked_icon_color: Option<Color>,
    thumb_icon_color: Option<Color>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    #[prop(skip_setter)] controller: Option<State<SwitchController>>,
    child: Option<RenderSlot>,
) {
    let checked = checked.unwrap_or(false);
    let controller = controller.unwrap_or_else(|| remember(|| SwitchController::new(checked)));
    render_switch(
        modifier,
        on_toggle,
        enabled,
        width,
        height,
        track_color,
        track_checked_color,
        track_outline_color,
        track_outline_width,
        thumb_color,
        thumb_checked_color,
        thumb_checked_icon_color,
        thumb_icon_color,
        accessibility_label,
        accessibility_description,
        controller,
        child,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_switch(
    modifier: Option<Modifier>,
    on_toggle: Option<CallbackWith<bool, ()>>,
    enabled: Option<bool>,
    width: Option<Dp>,
    height: Option<Dp>,
    track_color: Option<Color>,
    track_checked_color: Option<Color>,
    track_outline_color: Option<Color>,
    track_outline_width: Option<Dp>,
    thumb_color: Option<Color>,
    thumb_checked_color: Option<Color>,
    thumb_checked_icon_color: Option<Color>,
    thumb_icon_color: Option<Color>,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    controller: State<SwitchController>,
    child: Option<RenderSlot>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let enabled = enabled.unwrap_or(true);
    let width = width.unwrap_or(SwitchDefaults::WIDTH);
    let height = height.unwrap_or(SwitchDefaults::HEIGHT);
    let track_color = track_color.unwrap_or(scheme.surface_container_highest);
    let track_checked_color = track_checked_color.unwrap_or(scheme.primary);
    let track_outline_color = track_outline_color.unwrap_or(scheme.outline);
    let track_outline_width = track_outline_width.unwrap_or(SwitchDefaults::TRACK_OUTLINE_WIDTH);
    let thumb_color = thumb_color.unwrap_or(scheme.outline);
    let thumb_checked_color = thumb_checked_color.unwrap_or(scheme.on_primary);
    let thumb_checked_icon_color = thumb_checked_icon_color.unwrap_or(scheme.on_primary_container);
    let thumb_icon_color = thumb_icon_color.unwrap_or(scheme.surface_container_highest);

    let mut modifier = modifier.unwrap_or_default();

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

    let on_toggle = enabled.then_some(on_toggle).flatten();
    let interactive = on_toggle.is_some();
    let interaction_state = interactive.then(|| remember(InteractionState::new));
    let ripple_state = interactive.then(|| remember(RippleState::new));
    let checked = controller.with(|c| c.is_checked());
    if interactive {
        modifier = modifier.minimum_interactive_component_size();
        let ripple_spec = RippleSpec {
            bounded: false,
            radius: Some(Dp(SwitchDefaults::STATE_LAYER_SIZE.0 / 2.0)),
        };
        let ripple_size = PxSize::new(
            SwitchDefaults::STATE_LAYER_SIZE.to_px(),
            SwitchDefaults::STATE_LAYER_SIZE.to_px(),
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
            value: checked,
            on_value_change: CallbackWith::new(move |_| {
                controller.with_mut(|c| c.toggle());
                let checked = controller.with(|c| c.is_checked());
                if let Some(on_toggle) = on_toggle.as_ref() {
                    on_toggle.call(checked);
                }
            }),
            enabled,
            role: Some(Role::Switch),
            label: accessibility_label.clone(),
            description: accessibility_description.clone(),
            interaction_state,
            on_press: press_handler.map(Into::into),
            on_release: release_handler.map(Into::into),
            ..Default::default()
        };
        modifier = modifier.toggleable_with(toggle_args);
    }

    let has_thumb_content = child.is_some();
    let progress = controller.with(|c| c.animation_progress());
    let eased_progress = animation::easing(progress);
    let eased_progress_f64 = eased_progress as f64;
    let is_pressed = interaction_state
        .map(|state| state.with(|s| s.is_pressed()))
        .unwrap_or(false);
    let colors = SwitchDefaults::resolve_colors(
        SwitchColorInputs {
            track_color,
            track_checked_color,
            track_outline_color,
            thumb_color,
            thumb_checked_color,
            thumb_checked_icon_color,
            thumb_icon_color,
        },
        &scheme,
        SwitchStateFlags { checked, enabled },
    );

    let off_diameter = if has_thumb_content {
        SwitchDefaults::THUMB_DIAMETER
    } else {
        SwitchDefaults::UNCHECKED_THUMB_DIAMETER
    };
    let thumb_diameter_dp = if is_pressed {
        SwitchDefaults::PRESSED_THUMB_DIAMETER
    } else {
        Dp(off_diameter.0
            + (SwitchDefaults::THUMB_DIAMETER.0 - off_diameter.0) * eased_progress_f64)
    };
    let thumb_size_px = thumb_diameter_dp.to_px();
    layout()
        .modifier(modifier)
        .layout_policy(SwitchLayout {
            track_width: width,
            track_height: height,
            track_outline_width,
            thumb_diameter: thumb_diameter_dp,
            progress: eased_progress_f64,
            checked,
            is_pressed,
        })
        .child(move || {
            let inherited_content_color = use_context::<ContentColor>()
                .map(|c| c.get().current)
                .unwrap_or(ContentColor::default().current);

            let track_style = if checked {
                SurfaceStyle::Filled {
                    color: colors.track_color,
                }
            } else {
                SurfaceStyle::FilledOutlined {
                    fill_color: colors.track_color,
                    border_color: colors.track_outline_color,
                    border_width: track_outline_width,
                }
            };

            surface()
                .modifier(Modifier::new().size(width, height))
                .style(track_style)
                .shape(Shape::CAPSULE)
                .show_state_layer(false)
                .show_ripple(false)
                .with_child(|| {});

            let state_layer = surface()
                .modifier(Modifier::new().size(
                    SwitchDefaults::STATE_LAYER_SIZE,
                    SwitchDefaults::STATE_LAYER_SIZE,
                ))
                .shape(Shape::Ellipse)
                .style(SurfaceStyle::Filled {
                    color: Color::TRANSPARENT,
                })
                .show_state_layer(true)
                .show_ripple(true)
                .ripple_bounded(false)
                .ripple_radius(Dp(SwitchDefaults::STATE_LAYER_SIZE.0 / 2.0))
                .ripple_color(inherited_content_color);
            if let Some(interaction_state) = interaction_state {
                if let Some(ripple_state) = ripple_state {
                    state_layer
                        .interaction_state(interaction_state)
                        .ripple_state(ripple_state)
                        .with_child(|| {});
                } else {
                    state_layer
                        .interaction_state(interaction_state)
                        .with_child(|| {});
                }
            } else if let Some(ripple_state) = ripple_state {
                state_layer.ripple_state(ripple_state).with_child(|| {});
            } else {
                state_layer.with_child(|| {});
            }
            layout()
                .layout_policy(SwitchThumbLayout {
                    size: thumb_size_px,
                })
                .modifier({
                    let modifier = Modifier::new();
                    #[cfg(test)]
                    let modifier = modifier.semantics(crate::modifier::SemanticsArgs {
                        test_tag: Some(THUMB_TEST_TAG.to_string()),
                        ..Default::default()
                    });
                    modifier
                })
                .child(move || {
                    surface()
                        .modifier(Modifier::new().fill_max_size())
                        .style(SurfaceStyle::Filled {
                            color: colors.thumb_color,
                        })
                        .shape(Shape::Ellipse)
                        .content_color(colors.icon_color)
                        .with_child(move || {
                            if let Some(child) = child {
                                boxed()
                                    .modifier(Modifier::new().fill_max_size())
                                    .alignment(Alignment::Center)
                                    .children(move || {
                                        child.render();
                                    });
                            }
                        });
                });
        });
}

#[cfg(test)]
mod tests {
    use tessera_ui::{
        ComputedData, LayoutPolicy, LayoutResult, MeasurementError, Modifier, NoopRenderPolicy, Px,
        PxPosition,
        layout::{MeasureScope, layout},
        receive_frame_nanos, remember, tessera,
    };

    use crate::modifier::{ModifierExt as _, SemanticsArgs};
    use crate::theme::{MaterialTheme, material_theme};

    use super::{ANIMATION_DURATION, SwitchController, THUMB_TEST_TAG, render_switch};

    #[derive(Clone, PartialEq)]
    struct OffsetChildPolicy {
        x: i32,
    }

    impl LayoutPolicy for OffsetChildPolicy {
        fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
            let mut result = LayoutResult::default();
            let child_constraint = input.parent_constraint().without_min();
            let child = input
                .children()
                .first()
                .copied()
                .expect("offset child policy requires a child");
            let child_size = child.measure(&child_constraint)?;
            result.place_child(child, PxPosition::new(Px::new(self.x), Px::ZERO));

            Ok(result.with_size(ComputedData {
                width: Px::new(self.x + child_size.width.raw()),
                height: child_size.height,
            }))
        }
    }

    #[derive(Clone, PartialEq)]
    struct FixedSizePolicy {
        width: i32,
        height: i32,
    }

    impl LayoutPolicy for FixedSizePolicy {
        fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
            let mut result = LayoutResult::default();
            let child_constraint = input.parent_constraint().without_min();
            for child in input.children() {
                let _ = child.measure(&child_constraint)?;
                result.place_child(child, PxPosition::ZERO);
            }

            Ok(result.with_size(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            }))
        }
    }

    #[tessera]
    fn tagged_probe(tag: Option<String>, width: Option<i32>, height: Option<i32>) {
        let tag = tag.unwrap_or_default();
        let width = width.unwrap_or_default();
        let height = height.unwrap_or_default();

        layout()
            .layout_policy(FixedSizePolicy { width, height })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn animated_switch_progress_probe() {
        let controller = remember(|| SwitchController::new(false));
        let started = remember(|| false);

        if !started.get() {
            controller.with_mut(|c| c.toggle());
            started.set(true);
        }

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

        let offset = controller.with(|c| (c.animation_progress() * 100.0).round() as i32);
        layout()
            .layout_policy(OffsetChildPolicy { x: offset })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new())
            .child(|| {
                tagged_probe()
                    .tag("progress_probe".to_string())
                    .width(20)
                    .height(20);
            });
    }

    #[tessera]
    fn animated_switch_thumb_probe() {
        let controller = remember(|| SwitchController::new(false));
        let started = remember(|| false);

        if !started.get() {
            controller.with_mut(|c| c.toggle());
            started.set(true);
        }

        material_theme()
            .theme(MaterialTheme::default)
            .child(move || {
                render_switch(
                    None,
                    None,
                    Some(true),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    controller,
                    None,
                );
            });
    }

    #[test]
    fn switch_controller_animates_to_checked_state() {
        let mut controller = SwitchController::new(false);

        controller.toggle();

        assert!(controller.is_checked());
        assert!(controller.is_animating());
        assert_eq!(controller.animation_progress(), 0.0);

        let half_nanos = (ANIMATION_DURATION.as_nanos() / 2) as u64;
        controller.update_progress(half_nanos);
        let mid = controller.animation_progress();
        assert!(mid > 0.0 && mid < 1.0, "mid animation progress was {mid}");
        assert!(controller.is_animating());

        controller.update_progress(ANIMATION_DURATION.as_nanos() as u64);
        assert_eq!(controller.animation_progress(), 1.0);
        assert!(!controller.is_animating());
    }

    #[test]
    fn switch_controller_animates_to_unchecked_state() {
        let mut controller = SwitchController::new(true);

        controller.toggle();

        assert!(!controller.is_checked());
        assert!(controller.is_animating());
        assert_eq!(controller.animation_progress(), 1.0);

        let half_nanos = (ANIMATION_DURATION.as_nanos() / 2) as u64;
        controller.update_progress(half_nanos);
        let mid = controller.animation_progress();
        assert!(mid > 0.0 && mid < 1.0, "mid animation progress was {mid}");
        assert!(controller.is_animating());

        controller.update_progress(ANIMATION_DURATION.as_nanos() as u64);
        assert_eq!(controller.animation_progress(), 0.0);
        assert!(!controller.is_animating());
    }

    #[test]
    fn switch_animation_driver_advances_across_frames() {
        tessera_ui::assert_layout! {
            viewport: (200, 100),
            content: {
                animated_switch_progress_probe();
            },
            expect: {
                0 => {
                    node("progress_probe").position(0, 0).size(20, 20);
                },
                75_000_000 => {
                    node("progress_probe").position(50, 0).size(20, 20);
                },
                150_000_000 => {
                    node("progress_probe").position(100, 0).size(20, 20);
                }
            }
        }
    }

    #[test]
    fn switch_thumb_layout_grows_across_frames() {
        tessera_ui::assert_layout! {
            viewport: (200, 100),
            content: {
                animated_switch_thumb_probe();
            },
            expect: {
                0 => {
                    node(THUMB_TEST_TAG).size(16, 16);
                },
                75_000_000 => {
                    node(THUMB_TEST_TAG).size(20, 20);
                },
                150_000_000 => {
                    node(THUMB_TEST_TAG).size(24, 24);
                }
            }
        }
    }
}
