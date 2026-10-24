//! An interactive toggle switch component.
//!
//! ## Usage
//!
//! Use to control a boolean on/off state.
use tessera_foundation::gesture::TapRecognizer;
use tessera_ui::{
    AxisConstraint, CallbackWith, Color, ComputedData, Constraint, Dp, FocusRequester,
    LayoutResult, MeasurementError, Modifier, Px, PxPosition, PxSize, RenderSlot, State,
    accesskit::{Role, Toggled},
    current_frame_nanos,
    layout::{MeasureScope, PlacementResult, PlacementScope, layout},
    modifier::ModifierCapabilityExt as _,
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    animation::material_motion::{Scalar, Spec},
    boxed::boxed,
    icon::IconSize,
    modifier::{
        DisabledToggleSemantics, InteractionState, InteractionVisualState, ModifierExt,
        PointerEventContext, SemanticsArgs, ToggleableArgs,
    },
    ripple_state::{RippleSpec, RippleState},
    shape_def::Shape,
    surface::{SurfaceStyle, surface},
    theme::{MaterialAlpha, MaterialColorScheme, MaterialTheme},
};

use motion::Geometry;

mod motion;

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
    /// Default size of icons rendered inside the thumb.
    pub const ICON_SIZE: Dp = Dp(16.0);

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

#[derive(Clone, PartialEq)]
struct SwitchLayout {
    track_width: Dp,
    track_height: Dp,
    thumb_diameter: Dp,
    thumb_offset: Dp,
}

impl tessera_ui::LayoutModifierNode for SwitchLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let track = children[0];
        let state_layer = children[1];
        let thumb = children[2];
        let thumb_constraint = Constraint::NONE;
        let track_size = track.measure(input, &thumb_constraint)?;
        let state_layer_size = state_layer.measure(input, &thumb_constraint)?;
        let thumb_size = thumb.measure(input, &thumb_constraint)?;

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

        let thumb_x = self.thumb_offset.to_px();

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
            && self.thumb_diameter == other.thumb_diameter
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.track_width == other.track_width
            && self.track_height == other.track_height
            && self.thumb_diameter == other.thumb_diameter
            && self.thumb_offset == other.thumb_offset
    }

    fn place_children(
        &self,
        input: &PlacementScope<'_>,
        _inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        let mut result = LayoutResult::default();
        let children = input.children();
        if children.len() < 3 {
            return Some(result.into_placement_result());
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

        let thumb_x = self.thumb_offset.to_px();

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
        Some(result.into_placement_result())
    }
}

#[derive(Clone, PartialEq)]
struct SwitchThumbLayout {
    size: Px,
}

impl tessera_ui::LayoutModifierNode for SwitchThumbLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let constraint = Constraint::new(
            AxisConstraint::exact(self.size),
            AxisConstraint::exact(self.size),
        );

        for child in input.children() {
            let _ = child.measure(input, &constraint)?;
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
    progress: Scalar,
    snap_revision: u64,
}

impl SwitchController {
    /// Creates a new controller with the given initial value.
    pub fn new(initial_state: bool) -> Self {
        Self {
            checked: initial_state,
            progress: Scalar::new(if initial_state { 1.0 } else { 0.0 }),
            snap_revision: 0,
        }
    }

    /// Returns whether the switch is currently checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Sets the checked state immediately, finishing any ongoing transition.
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
        self.progress.snap(if checked { 1.0 } else { 0.0 });
        self.snap_revision = self.snap_revision.wrapping_add(1);
    }

    /// Toggles the switch with an interruptible transition.
    pub fn toggle(&mut self) {
        self.toggle_at(current_frame_nanos());
    }

    fn toggle_at(&mut self, frame_nanos: u64) {
        self.checked = !self.checked;
        self.progress.retarget(
            if self.checked { 1.0 } else { 0.0 },
            Spec::FastSpatial,
            frame_nanos,
        );
    }

    /// Returns the current selection progress, clamped to 0.0..1.0.
    pub fn animation_progress(&self) -> f32 {
        self.progress.value.clamp(0.0, 1.0) as f32
    }

    /// Returns whether the selection transition is still settling.
    pub fn is_animating(&self) -> bool {
        self.progress.is_animating()
    }

    fn update_progress(&mut self, frame_nanos: u64) {
        self.progress.advance(frame_nanos);
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
/// - `focus_requester` — optional handle targeting the switch for focus
///   requests.
/// - `on_toggle` — optional callback invoked when the switch toggles; omit for
///   a noninteractive state display that retains its accessibility value.
/// - `enabled` — optional enabled state; defaults to `true`.
/// - `presentation_state` — optional visual interaction state; live input and
///   disabled state take precedence.
/// - `checked` — externally controlled checked state; omit to use the
///   controller.
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
/// - `child` — optional content rendered at the thumb center; icons default to
///   16dp and inherit the current thumb icon color.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::switch::{SwitchController, switch};
/// use tessera_ui::remember;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// material_theme()
///     .theme(|| MaterialTheme::default())
///     .child(|| {
///         let controller = remember(|| {
///             let mut controller = SwitchController::new(false);
///             controller.toggle();
///             assert!(controller.is_checked());
///             controller
///         });
///         switch().controller(controller).on_toggle(|_| {});
///     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn switch(
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    on_toggle: Option<CallbackWith<bool, ()>>,
    enabled: Option<bool>,
    presentation_state: Option<InteractionVisualState>,
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
    controller: Option<State<SwitchController>>,
    child: Option<RenderSlot>,
) {
    let controller = if let Some(controller) = controller {
        controller
    } else {
        remember(|| SwitchController::new(checked.unwrap_or(false)))
    };
    if let Some(checked) = checked
        && controller.with(|controller| controller.is_checked()) != checked
    {
        controller.with_mut(|controller| controller.toggle());
    }
    let focus_requester = if let Some(focus_requester) = focus_requester {
        focus_requester
    } else {
        remember(FocusRequester::new).get()
    };
    let enabled = enabled.unwrap_or(true);
    let interactive = enabled && on_toggle.is_some();
    let interaction_state = if interactive {
        Some(remember(InteractionState::new))
    } else {
        None
    };
    let ripple_state = if interactive {
        Some(remember(RippleState::new))
    } else {
        None
    };
    let tap_recognizer = if interactive {
        Some(remember(TapRecognizer::default))
    } else {
        None
    };

    if controller.with(|c| c.is_animating()) {
        receive_frame_nanos(move |frame_nanos| {
            #[cfg(test)]
            motion_tests::TICKS.set(motion_tests::TICKS.get() + 1);
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

    switch_content()
        .focus_requester(focus_requester)
        .modifier_optional(modifier)
        .on_toggle_optional(on_toggle)
        .enabled(enabled)
        .presentation_state_optional(presentation_state)
        .width_optional(width)
        .height_optional(height)
        .track_color_optional(track_color)
        .track_checked_color_optional(track_checked_color)
        .track_outline_color_optional(track_outline_color)
        .track_outline_width_optional(track_outline_width)
        .thumb_color_optional(thumb_color)
        .thumb_checked_color_optional(thumb_checked_color)
        .thumb_checked_icon_color_optional(thumb_checked_icon_color)
        .thumb_icon_color_optional(thumb_icon_color)
        .accessibility_label_optional(accessibility_label)
        .accessibility_description_optional(accessibility_description)
        .controller(controller)
        .interaction_state_optional(interaction_state)
        .ripple_state_optional(ripple_state)
        .tap_recognizer_optional(tap_recognizer)
        .child_optional(child);
}

#[tessera]
#[allow(clippy::too_many_arguments)]
fn switch_content(
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    on_toggle: Option<CallbackWith<bool, ()>>,
    enabled: Option<bool>,
    presentation_state: Option<InteractionVisualState>,
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
    controller: Option<State<SwitchController>>,
    interaction_state: Option<State<InteractionState>>,
    ripple_state: Option<State<RippleState>>,
    tap_recognizer: Option<State<TapRecognizer>>,
    child: Option<RenderSlot>,
) {
    let enabled = enabled.unwrap_or(true);
    let controller = controller.expect("switch_content requires controller to be set");
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let width = width.unwrap_or(SwitchDefaults::WIDTH);
    let height = height.unwrap_or(SwitchDefaults::HEIGHT);
    let track_color = track_color.unwrap_or(scheme.surface_container_highest);
    let track_checked_color = track_checked_color.unwrap_or(scheme.primary);
    let track_outline_color = track_outline_color.unwrap_or(scheme.outline);
    let track_outline_width = track_outline_width.unwrap_or(SwitchDefaults::TRACK_OUTLINE_WIDTH);
    let visual = interaction_state
        .map(|state| state.with(|state| state.with_presentation(presentation_state, enabled)))
        .unwrap_or_else(|| InteractionState::new().with_presentation(presentation_state, enabled));
    let interacting = visual.is_hovered() || visual.is_focused() || visual.is_pressed();
    let thumb_color = thumb_color.unwrap_or(if interacting {
        scheme.on_surface_variant
    } else {
        scheme.outline
    });
    let thumb_checked_color = thumb_checked_color.unwrap_or(if interacting {
        scheme.primary_container
    } else {
        scheme.on_primary
    });
    let thumb_checked_icon_color = thumb_checked_icon_color.unwrap_or(scheme.on_primary_container);
    let thumb_icon_color = thumb_icon_color.unwrap_or(scheme.surface_container_highest);

    let mut modifier = modifier.unwrap_or_default();

    let interactive = on_toggle.is_some();
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
            focus_requester,
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
            tap_recognizer,
            ..Default::default()
        };
        modifier = if enabled {
            modifier.toggleable_with(toggle_args)
        } else {
            modifier.push_semantics(DisabledToggleSemantics {
                role: Role::Switch,
                value: checked,
                label: accessibility_label,
                description: accessibility_description,
            })
        };
    } else {
        modifier = modifier.semantics(SemanticsArgs {
            role: Some(Role::Switch),
            toggled: Some(if checked {
                Toggled::True
            } else {
                Toggled::False
            }),
            label: accessibility_label,
            description: accessibility_description,
            disabled: !enabled,
            ..Default::default()
        });
    }

    let has_thumb_content = child.is_some();
    let is_pressed = visual.is_pressed();
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

    let target_diameter = if is_pressed {
        SwitchDefaults::PRESSED_THUMB_DIAMETER
    } else if has_thumb_content || checked {
        SwitchDefaults::THUMB_DIAMETER
    } else {
        SwitchDefaults::UNCHECKED_THUMB_DIAMETER
    };
    let target_offset = if checked {
        width.0
            - SwitchDefaults::THUMB_DIAMETER.0
            - (height.0 - SwitchDefaults::THUMB_DIAMETER.0) / 2.0
            - if is_pressed {
                track_outline_width.0
            } else {
                0.0
            }
    } else if is_pressed {
        track_outline_width.0
    } else {
        (height.0 - target_diameter.0) / 2.0
    };
    let revision = controller.with(|controller| controller.snap_revision);
    let geometry = remember(|| Geometry::new(target_offset, target_diameter.0, revision));
    let mut current_geometry = geometry.get();
    current_geometry.retarget(
        target_offset,
        target_diameter.0,
        is_pressed,
        revision,
        current_frame_nanos(),
    );
    if geometry.with(|geometry| *geometry != current_geometry) {
        geometry.set(current_geometry);
    }
    if geometry.with(Geometry::is_animating) {
        receive_frame_nanos(move |frame| {
            #[cfg(test)]
            motion_tests::TICKS.set(motion_tests::TICKS.get() + 1);
            let active = geometry.with_mut(|geometry| {
                geometry.advance(frame);
                geometry.is_animating()
            });
            if active {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let thumb_diameter_dp = Dp(geometry.with(|geometry| geometry.diameter.value).max(0.0));
    let thumb_offset = Dp(geometry.with(|geometry| geometry.offset.value));
    let thumb_size_px = thumb_diameter_dp.to_px();
    layout()
        .modifier(modifier)
        .policy(SwitchLayout {
            track_width: width,
            track_height: height,
            thumb_diameter: thumb_diameter_dp,
            thumb_offset,
        })
        .child(move || {
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
                .focus_requester_optional(focus_requester)
                .interaction_state_optional(interaction_state)
                .presentation_state_optional(presentation_state)
                .show_focus_indicator(true)
                .enabled(enabled)
                .child(|| {});

            surface()
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
                .ripple_color(if checked {
                    scheme.primary
                } else {
                    scheme.on_surface
                })
                .interaction_state_optional(interaction_state)
                .presentation_state_optional(presentation_state)
                .enabled(enabled)
                .ripple_state_optional(ripple_state)
                .child(|| {});
            layout()
                .policy(SwitchThumbLayout {
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
                        .child(move || {
                            if let Some(child) = child {
                                boxed()
                                    .modifier(Modifier::new().fill_max_size())
                                    .alignment(Alignment::Center)
                                    .children(move || {
                                        provide_context(
                                            || IconSize(SwitchDefaults::ICON_SIZE),
                                            move || child.render(),
                                        );
                                    });
                            }
                        });
                });
        });
}

#[cfg(test)]
mod state_tests;

#[cfg(test)]
mod motion_tests;

#[cfg(test)]
mod tests {
    use super::SwitchController;

    #[test]
    fn selection_spring_matches_reference_samples_and_settles_both_values() {
        for checked in [false, true] {
            let mut controller = SwitchController::new(checked);
            controller.toggle_at(0);
            controller.update_progress(50_000_000);
            let expected = if checked {
                1.0 - 0.5273674995
            } else {
                0.5273674995
            };
            assert!((controller.animation_progress() as f64 - expected).abs() < 0.000001);
            controller.update_progress(150_000_000);
            assert!(if checked {
                controller.progress.value < 0.0
            } else {
                controller.progress.value > 1.0
            });
            assert!(controller.is_animating());
            controller.update_progress(2_000_000_000);
            assert_eq!(
                controller.animation_progress(),
                if checked { 0.0 } else { 1.0 }
            );
            assert!(!controller.is_animating());
        }
    }

    #[test]
    fn retarget_samples_the_incoming_position_and_preserves_velocity() {
        let mut controller = SwitchController::new(false);
        controller.toggle_at(0);
        controller.update_progress(50_000_000);
        controller.toggle_at(75_000_000);
        assert!((controller.progress.value - 0.8269005484).abs() < 0.000001);
        assert!((controller.progress.velocity - 9.8223966780).abs() < 0.000001);
        controller.update_progress(76_000_000);
        assert!(controller.progress.value > 0.8269005484);
        controller.update_progress(2_000_000_000);
        assert_eq!(controller.animation_progress(), 0.0);
        assert!(!controller.is_animating());
    }

    #[test]
    fn setting_the_current_target_finishes_an_active_transition() {
        let mut controller = SwitchController::new(false);
        controller.toggle_at(0);
        controller.update_progress(50_000_000);
        controller.set_checked(true);
        assert_eq!(controller.animation_progress(), 1.0);
        assert!(!controller.is_animating());
        controller.set_checked(false);
        assert_eq!(controller.animation_progress(), 0.0);
        assert!(!controller.is_animating());
    }
}
