//! An interactive toggle switch component.
//!
//! ## Usage
//!
//! Use to control a boolean on/off state.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, GestureState,
    PressKeyEventType, PxPosition, PxSize, State,
    accesskit::{Action, Role, Toggled},
    remember, tessera, use_context,
    winit::window::CursorIcon,
};

use crate::{
    alignment::Alignment,
    animation,
    boxed::{BoxedArgsBuilder, boxed},
    ripple_state::{RippleSpec, RippleState},
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, SurfaceStyle, surface},
    theme::{ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme},
};

const ANIMATION_DURATION: Duration = Duration::from_millis(150);

/// Material Design 3 defaults for [`switch`].
pub struct SwitchDefaults;

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
        args: &SwitchArgs,
        scheme: &MaterialColorScheme,
        checked: bool,
        enabled: bool,
    ) -> SwitchResolvedColors {
        let mut track_color = if checked {
            args.track_checked_color
        } else {
            args.track_color
        };
        let mut track_outline_color = if checked {
            Color::TRANSPARENT
        } else {
            args.track_outline_color
        };
        let mut thumb_color = if checked {
            args.thumb_checked_color
        } else {
            args.thumb_color
        };
        let mut icon_color = if checked {
            args.thumb_checked_icon_color
        } else {
            args.thumb_icon_color
        };

        if !enabled {
            let disabled_container_alpha = MaterialAlpha::DISABLED_CONTAINER;
            let disabled_content_alpha = MaterialAlpha::DISABLED_CONTENT;

            if checked {
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

/// Controller for the `switch` component.
pub struct SwitchController {
    checked: bool,
    progress: f32,
    last_toggle_time: Option<Instant>,
}

impl SwitchController {
    /// Creates a new controller with the given initial value.
    pub fn new(initial_state: bool) -> Self {
        Self {
            checked: initial_state,
            progress: if initial_state { 1.0 } else { 0.0 },
            last_toggle_time: None,
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
            self.last_toggle_time = None;
        }
    }

    /// Toggles the switch and kicks off the animation timeline.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
        self.last_toggle_time = Some(Instant::now());
    }

    /// Returns the current animation progress (0.0..1.0).
    pub fn animation_progress(&self) -> f32 {
        self.progress
    }

    /// Advances the animation timeline based on elapsed time.
    fn update_progress(&mut self) {
        if let Some(last_toggle_time) = self.last_toggle_time {
            let elapsed = last_toggle_time.elapsed();
            let fraction = (elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32()).min(1.0);
            let target = if self.checked { 1.0 } else { 0.0 };
            let progress = if self.checked {
                fraction
            } else {
                1.0 - fraction
            };

            self.progress = progress;

            if (progress - target).abs() < f32::EPSILON || fraction >= 1.0 {
                self.progress = target;
                self.last_toggle_time = None;
            }
        }
    }
}

impl Default for SwitchController {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Arguments for configuring the `switch` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct SwitchArgs {
    /// Optional callback invoked when the switch toggles.
    #[builder(default, setter(strip_option))]
    pub on_toggle: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    /// Whether the control is enabled for user interaction.
    ///
    /// When `false`, the switch will not react to input and will expose a
    /// disabled state to accessibility services.
    #[builder(default = "true")]
    pub enabled: bool,
    /// Initial checked state.
    #[builder(default = "false")]
    pub checked: bool,
    /// Total width of the switch track.
    #[builder(default = "SwitchDefaults::WIDTH")]
    pub width: Dp,
    /// Total height of the switch track (including padding).
    #[builder(default = "SwitchDefaults::HEIGHT")]
    pub height: Dp,
    /// Track color when the switch is off.
    #[builder(
        default = "use_context::<MaterialTheme>().get().color_scheme.surface_container_highest"
    )]
    pub track_color: Color,
    /// Track color when the switch is on.
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.primary")]
    pub track_checked_color: Color,
    /// Outline color for the track when the switch is off.
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.outline")]
    pub track_outline_color: Color,
    /// Border width for the track outline.
    #[builder(default = "SwitchDefaults::TRACK_OUTLINE_WIDTH")]
    pub track_outline_width: Dp,
    /// Thumb color when the switch is off.
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.outline")]
    pub thumb_color: Color,
    /// Thumb color when the switch is on.
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.on_primary")]
    pub thumb_checked_color: Color,
    /// Icon color when the switch is on.
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.on_primary_container")]
    pub thumb_checked_icon_color: Color,
    /// Icon color when the switch is off.
    #[builder(
        default = "use_context::<MaterialTheme>().get().color_scheme.surface_container_highest"
    )]
    pub thumb_icon_color: Color,
    /// Optional accessibility label read by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
}

impl Default for SwitchArgs {
    fn default() -> Self {
        SwitchArgsBuilder::default()
            .build()
            .expect("builder construction failed")
    }
}

fn is_cursor_in_component(size: ComputedData, pos_option: Option<tessera_ui::PxPosition>) -> bool {
    pos_option
        .map(|pos| {
            pos.x.0 >= 0 && pos.x.0 < size.width.0 && pos.y.0 >= 0 && pos.y.0 < size.height.0
        })
        .unwrap_or(false)
}

fn handle_input_events_switch(
    controller: State<SwitchController>,
    on_toggle: &Option<Arc<dyn Fn(bool) + Send + Sync>>,
    pressed: State<bool>,
    interaction_state: Option<State<RippleState>>,
    input: &mut tessera_ui::InputHandlerInput,
) {
    controller.with_mut(|c| c.update_progress());

    let size = input.computed_data;
    let is_cursor_in = is_cursor_in_component(size, input.cursor_position_rel);
    let interactive = on_toggle.is_some();

    if interactive && let Some(interaction_state) = interaction_state {
        interaction_state.with_mut(|s| s.set_hovered(is_cursor_in));
    }

    if is_cursor_in && interactive {
        input.requests.cursor_icon = CursorIcon::Pointer;
    }

    for e in input.cursor_events.iter() {
        if matches!(
            e.content,
            CursorEventContent::Pressed(PressKeyEventType::Left)
        ) && is_cursor_in
        {
            if interactive && let Some(interaction_state) = interaction_state {
                pressed.set(true);
                interaction_state.with_mut(|s| {
                    s.start_animation_with_spec(
                        [0.5, 0.5],
                        PxSize::new(
                            SwitchDefaults::STATE_LAYER_SIZE.to_px(),
                            SwitchDefaults::STATE_LAYER_SIZE.to_px(),
                        ),
                        RippleSpec {
                            bounded: false,
                            radius: Some(Dp(SwitchDefaults::STATE_LAYER_SIZE.0 / 2.0)),
                        },
                    );
                    s.set_pressed(true);
                });
            } else if interactive {
                pressed.set(true);
            }
        }
        if matches!(
            e.content,
            CursorEventContent::Released(PressKeyEventType::Left)
        ) && interactive
        {
            pressed.set(false);
            if let Some(interaction_state) = interaction_state {
                interaction_state.with_mut(|s| s.release());
            }
        }
        if interactive
            && e.gesture_state == GestureState::TapCandidate
            && matches!(
                e.content,
                CursorEventContent::Released(PressKeyEventType::Left)
            )
            && is_cursor_in
        {
            toggle_switch_state(controller, on_toggle);
        }
    }

    if !is_cursor_in && interactive {
        pressed.set(false);
        if let Some(interaction_state) = interaction_state {
            interaction_state.with_mut(|s| {
                s.release();
                s.set_hovered(false);
            });
        }
    }
}

fn toggle_switch_state(
    controller: State<SwitchController>,
    on_toggle: &Option<Arc<dyn Fn(bool) + Send + Sync>>,
) -> bool {
    let Some(on_toggle) = on_toggle else {
        return false;
    };

    controller.with_mut(|c| c.toggle());
    let checked = controller.with(|c| c.is_checked());
    on_toggle(checked);
    true
}

fn apply_switch_accessibility(
    input: &mut tessera_ui::InputHandlerInput<'_>,
    controller: State<SwitchController>,
    enabled: bool,
    on_toggle: &Option<Arc<dyn Fn(bool) + Send + Sync>>,
    label: Option<&String>,
    description: Option<&String>,
) {
    let checked = controller.with(|c| c.is_checked());
    let mut builder = input.accessibility().role(Role::Switch);

    if let Some(label) = label {
        builder = builder.label(label.clone());
    }
    if let Some(description) = description {
        builder = builder.description(description.clone());
    }

    builder = builder.toggled(if checked {
        Toggled::True
    } else {
        Toggled::False
    });

    if !enabled {
        builder = builder.disabled();
    } else if on_toggle.is_some() {
        builder = builder.focusable().action(Action::Click);
    }

    builder.commit();

    if enabled && on_toggle.is_some() {
        let on_toggle = on_toggle.clone();
        input.set_accessibility_action_handler(move |action| {
            if action == Action::Click {
                toggle_switch_state(controller, &on_toggle);
            }
        });
    }
}

#[tessera]
fn switch_inner(
    args: SwitchArgs,
    controller: State<SwitchController>,
    child: Option<Box<dyn FnOnce() + Send + Sync>>,
) {
    let pressed = remember(|| false);
    controller.with_mut(|c| c.update_progress());

    let has_thumb_content = child.is_some();
    let is_pressed = pressed.with(|v| *v);
    let progress = controller.with(|c| c.animation_progress());
    let eased_progress = animation::easing(progress);
    let eased_progress_f64 = eased_progress as f64;
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
    let checked = controller.with(|c| c.is_checked());
    let enabled = args.enabled;
    let on_toggle = enabled.then(|| args.on_toggle.clone()).flatten();
    let interactive = on_toggle.is_some();
    let interaction_state = interactive.then(|| remember(RippleState::new));
    let colors = SwitchDefaults::resolve_colors(&args, &scheme, checked, enabled);

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

    let inherited_content_color = use_context::<ContentColor>().get().current;

    let track_style = if checked {
        SurfaceStyle::Filled {
            color: colors.track_color,
        }
    } else {
        SurfaceStyle::FilledOutlined {
            fill_color: colors.track_color,
            border_color: colors.track_outline_color,
            border_width: args.track_outline_width,
        }
    };

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(args.width.to_px()))
            .height(DimensionValue::Fixed(args.height.to_px()))
            .style(track_style)
            .shape(Shape::capsule())
            .enforce_min_interactive_size(false)
            .show_state_layer(false)
            .show_ripple(false)
            .build()
            .expect("builder construction failed"),
        || {},
    );

    // A non-visual state layer for hover + ripple feedback.
    let mut state_layer_builder = SurfaceArgsBuilder::default()
        .width(DimensionValue::Fixed(
            SwitchDefaults::STATE_LAYER_SIZE.to_px(),
        ))
        .height(DimensionValue::Fixed(
            SwitchDefaults::STATE_LAYER_SIZE.to_px(),
        ))
        .shape(Shape::Ellipse)
        .enforce_min_interactive_size(false)
        .style(SurfaceStyle::Filled {
            color: Color::TRANSPARENT,
        })
        .show_state_layer(true)
        .show_ripple(true)
        .ripple_bounded(false)
        .ripple_radius(Dp(SwitchDefaults::STATE_LAYER_SIZE.0 / 2.0))
        .ripple_color(inherited_content_color);
    if let Some(interaction_state) = interaction_state {
        state_layer_builder = state_layer_builder.interaction_state(interaction_state);
    }
    surface(
        state_layer_builder
            .build()
            .expect("builder construction failed"),
        || {},
    );

    let child = child;
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(thumb_size_px))
            .height(DimensionValue::Fixed(thumb_size_px))
            .style(SurfaceStyle::Filled {
                color: colors.thumb_color,
            })
            .shape(Shape::Ellipse)
            .content_color(colors.icon_color)
            .build()
            .expect("builder construction failed"),
        move || {
            if let Some(child) = child {
                boxed(
                    BoxedArgsBuilder::default()
                        .width(DimensionValue::Fixed(thumb_size_px))
                        .height(DimensionValue::Fixed(thumb_size_px))
                        .alignment(Alignment::Center)
                        .build()
                        .expect("builder construction failed"),
                    |scope| {
                        scope.child(move || {
                            child();
                        });
                    },
                );
            }
        },
    );

    let accessibility_on_toggle = on_toggle.clone();
    let accessibility_label = args.accessibility_label.clone();
    let accessibility_description = args.accessibility_description.clone();
    let track_outline_width = args.track_outline_width;
    let track_width = args.width;
    let track_height = args.height;

    input_handler(Box::new(move |mut input| {
        // Delegate input handling to the extracted helper.
        handle_input_events_switch(
            controller,
            &on_toggle,
            pressed,
            interaction_state,
            &mut input,
        );
        apply_switch_accessibility(
            &mut input,
            controller,
            enabled,
            &accessibility_on_toggle,
            accessibility_label.as_ref(),
            accessibility_description.as_ref(),
        );
    }));

    measure(Box::new(move |input| {
        let track_id = input.children_ids[0];
        let state_layer_id = input.children_ids[1];
        let thumb_id = input.children_ids[2];
        let thumb_constraint = Constraint::new(
            DimensionValue::Wrap {
                min: None,
                max: None,
            },
            DimensionValue::Wrap {
                min: None,
                max: None,
            },
        );
        let track_size = input.measure_child(track_id, &thumb_constraint)?;
        let state_layer_size = input.measure_child(state_layer_id, &thumb_constraint)?;
        let thumb_size = input.measure_child(thumb_id, &thumb_constraint)?;

        let min_interactive_size = Dp(48.0).to_px();
        let self_width_px = if interactive {
            track_size.width.max(min_interactive_size)
        } else {
            track_size.width
        };
        let self_height_px = if interactive {
            track_size.height.max(min_interactive_size)
        } else {
            track_size.height
        };
        let track_origin_x = (self_width_px.0 - track_size.width.0) / 2;
        let track_origin_y = (self_height_px.0 - track_size.height.0) / 2;

        // Calculate thumb positioning:
        // - unchecked offset is (trackHeight - thumbDiameter) / 2
        // - checked offset is (trackWidth - checkedThumbDiameter) - thumbPadding
        // - pressed snaps towards the inside by TrackOutlineWidth
        let checked_thumb_diameter = SwitchDefaults::THUMB_DIAMETER;
        let thumb_padding_start = Dp((track_height.0 - checked_thumb_diameter.0) / 2.0);
        let max_bound_dp = Dp((track_width.0 - checked_thumb_diameter.0) - thumb_padding_start.0);

        let min_bound_dp = Dp((track_height.0 - thumb_diameter_dp.0) / 2.0);
        let anim_offset_dp =
            Dp(min_bound_dp.0 + (max_bound_dp.0 - min_bound_dp.0) * eased_progress_f64);
        let offset_dp = if is_pressed && checked {
            Dp(max_bound_dp.0 - track_outline_width.0)
        } else if is_pressed && !checked {
            track_outline_width
        } else {
            anim_offset_dp
        };
        let thumb_x = offset_dp.to_px();

        // State layer follows the thumb center.
        let thumb_center_x = track_origin_x + thumb_x.0 + thumb_size.width.0 / 2;
        let thumb_center_y = track_origin_y + track_size.height.0 / 2;
        let state_layer_x = thumb_center_x - state_layer_size.width.0 / 2;
        let state_layer_y = thumb_center_y - state_layer_size.height.0 / 2;

        input.place_child(
            track_id,
            PxPosition::new(
                tessera_ui::Px(track_origin_x),
                tessera_ui::Px(track_origin_y),
            ),
        );
        input.place_child(
            thumb_id,
            PxPosition::new(
                tessera_ui::Px(track_origin_x + thumb_x.0),
                tessera_ui::Px(track_origin_y + (track_size.height.0 - thumb_size.height.0) / 2),
            ),
        );
        input.place_child(
            state_layer_id,
            PxPosition::new(tessera_ui::Px(state_layer_x), tessera_ui::Px(state_layer_y)),
        );

        Ok(ComputedData {
            width: self_width_px,
            height: self_height_px,
        })
    }));
}

/// # switch
///
/// Convenience wrapper for `switch_with_child` that renders no thumb content.
///
/// ## Usage
///
/// Use when you want a standard on/off switch without a custom icon.
///
/// ## Parameters
///
/// - `args` — configures sizing, colors, and callbacks; see [`SwitchArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use std::sync::Arc;
/// use tessera_ui_basic_components::switch::{SwitchArgsBuilder, switch};
///
/// switch(
///     SwitchArgsBuilder::default()
///         .on_toggle(Arc::new(|checked| {
///             assert!(checked || !checked);
///         }))
///         .build()
///         .unwrap(),
/// );
/// # }
/// # component();
/// ```
#[tessera]
pub fn switch(args: impl Into<SwitchArgs>) {
    let args: SwitchArgs = args.into();
    let controller = remember(|| SwitchController::new(args.checked));
    switch_with_controller(args, controller);
}

/// # switch_with_child
///
/// Animated Material 3 switch with required custom thumb content; ideal for
/// boolean toggles that show an icon or label.
///
/// ## Usage
///
/// Use for settings or binary preferences; provide a `child` to draw custom
/// content (e.g., an icon) inside the thumb.
///
/// ## Parameters
///
/// - `args` — configures sizing, colors, and callbacks; see [`SwitchArgs`].
/// - `controller` — an `Arc<SwitchController>` to drive and observe state.
/// - `child` — closure rendered at the thumb center.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use std::sync::Arc;
/// use tessera_ui_basic_components::switch::{SwitchArgsBuilder, switch_with_child};
/// use tessera_ui_basic_components::text::{TextArgsBuilder, text};
///
/// switch_with_child(
///     SwitchArgsBuilder::default()
///         .on_toggle(Arc::new(|checked| {
///             assert!(checked || !checked);
///         }))
///         .build()
///         .unwrap(),
///     || {
///         text(
///             TextArgsBuilder::default()
///                 .text("✓".to_string())
///                 .build()
///                 .unwrap(),
///         );
///     },
/// );
/// # }
/// # component();
/// ```
#[tessera]
pub fn switch_with_child(
    args: impl Into<SwitchArgs>,
    child: impl FnOnce() + Send + Sync + 'static,
) {
    let args = args.into();
    let controller = remember(|| SwitchController::new(args.checked));
    switch_inner(args, controller, Some(Box::new(child)));
}

/// # switch_with_child_and_controller
///
/// Controlled switch variant with custom thumb content.
///
/// # Usage
///
/// Use when you need to sync the switch state with external application state
/// and want custom thumb content.
///
/// # Parameters
///
/// - `args` — configures sizing, colors, and callbacks; see [`SwitchArgs`].
/// - `controller` — an `Arc<SwitchController>` to drive and observe state.
/// - `child` — closure rendered at the thumb center.
///
/// # Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use std::sync::Arc;
/// use tessera_ui::{remember, tessera};
/// use tessera_ui_basic_components::switch::{
///     SwitchArgsBuilder, SwitchController, switch_with_child_and_controller,
/// };
/// use tessera_ui_basic_components::text::{TextArgsBuilder, text};
///
/// #[tessera]
/// fn controlled_switch_example() {
///     let controller = remember(|| SwitchController::new(false));
///     switch_with_child_and_controller(
///         SwitchArgsBuilder::default()
///             .on_toggle(Arc::new(|checked| {
///                 println!("Switch is now: {}", checked);
///             }))
///             .build()
///             .unwrap(),
///         controller,
///         || {
///             text(
///                 TextArgsBuilder::default()
///                     .text("✓".to_string())
///                     .build()
///                     .unwrap(),
///             );
///         },
///     );
/// }
/// # }
/// # component();
/// ```
#[tessera]
pub fn switch_with_child_and_controller(
    args: impl Into<SwitchArgs>,
    controller: State<SwitchController>,
    child: impl FnOnce() + Send + Sync + 'static,
) {
    switch_inner(args.into(), controller, Some(Box::new(child)));
}

/// Controlled switch variant without thumb content customization.
pub fn switch_with_controller(args: impl Into<SwitchArgs>, controller: State<SwitchController>) {
    switch_inner(args.into(), controller, None);
}
