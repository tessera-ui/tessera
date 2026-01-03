//! An interactive toggle switch component.
//!
//! ## Usage
//!
//! Use to control a boolean on/off state.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_setters::Setters;
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Modifier, Px,
    PxPosition, PxSize, State,
    accesskit::Role,
    layout::{LayoutInput, LayoutOutput, LayoutSpec},
    remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    animation,
    boxed::{BoxedArgs, boxed},
    modifier::{InteractionState, ModifierExt, PointerEventContext, ToggleableArgs},
    ripple_state::{RippleSpec, RippleState},
    shape_def::Shape,
    surface::{SurfaceArgs, SurfaceStyle, surface},
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

impl LayoutSpec for SwitchLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let track_id = input.children_ids()[0];
        let state_layer_id = input.children_ids()[1];
        let thumb_id = input.children_ids()[2];
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

        output.place_child(
            track_id,
            PxPosition::new(Px(track_origin_x), Px(track_origin_y)),
        );
        output.place_child(
            thumb_id,
            PxPosition::new(
                Px(track_origin_x + thumb_x.0),
                Px(track_origin_y + (track_size.height.0 - thumb_size.height.0) / 2),
            ),
        );
        output.place_child(
            state_layer_id,
            PxPosition::new(Px(state_layer_x), Px(state_layer_y)),
        );

        Ok(ComputedData {
            width: self_width_px,
            height: self_height_px,
        })
    }
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
#[derive(Clone, Setters)]
pub struct SwitchArgs {
    /// Optional modifier chain applied to the switch subtree.
    pub modifier: Modifier,
    /// Optional callback invoked when the switch toggles.
    #[setters(skip)]
    pub on_toggle: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    /// Whether the control is enabled for user interaction.
    ///
    /// When `false`, the switch will not react to input and will expose a
    /// disabled state to accessibility services.
    pub enabled: bool,
    /// Initial checked state.
    pub checked: bool,
    /// Total width of the switch track.
    pub width: Dp,
    /// Total height of the switch track (including padding).
    pub height: Dp,
    /// Track color when the switch is off.
    pub track_color: Color,
    /// Track color when the switch is on.
    pub track_checked_color: Color,
    /// Outline color for the track when the switch is off.
    pub track_outline_color: Color,
    /// Border width for the track outline.
    pub track_outline_width: Dp,
    /// Thumb color when the switch is off.
    pub thumb_color: Color,
    /// Thumb color when the switch is on.
    pub thumb_checked_color: Color,
    /// Icon color when the switch is on.
    pub thumb_checked_icon_color: Color,
    /// Icon color when the switch is off.
    pub thumb_icon_color: Color,
    /// Optional accessibility label read by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
}

impl SwitchArgs {
    /// Sets the on_toggle handler.
    pub fn on_toggle<F>(mut self, on_toggle: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_toggle = Some(Arc::new(on_toggle));
        self
    }

    /// Sets the on_toggle handler using a shared callback.
    pub fn on_toggle_shared(mut self, on_toggle: Arc<dyn Fn(bool) + Send + Sync>) -> Self {
        self.on_toggle = Some(on_toggle);
        self
    }
}

impl Default for SwitchArgs {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        Self {
            modifier: Modifier::new(),
            on_toggle: None,
            enabled: true,
            checked: false,
            width: SwitchDefaults::WIDTH,
            height: SwitchDefaults::HEIGHT,
            track_color: scheme.surface_container_highest,
            track_checked_color: scheme.primary,
            track_outline_color: scheme.outline,
            track_outline_width: SwitchDefaults::TRACK_OUTLINE_WIDTH,
            thumb_color: scheme.outline,
            thumb_checked_color: scheme.on_primary,
            thumb_checked_icon_color: scheme.on_primary_container,
            thumb_icon_color: scheme.surface_container_highest,
            accessibility_label: None,
            accessibility_description: None,
        }
    }
}

#[tessera]
fn switch_inner(
    args: SwitchArgs,
    controller: State<SwitchController>,
    child: Option<Box<dyn FnOnce() + Send + Sync>>,
) {
    let mut modifier = args.modifier;

    controller.with_mut(|c| c.update_progress());

    let on_toggle = args.enabled.then(|| args.on_toggle.clone()).flatten();
    let interactive = on_toggle.is_some();
    let interaction_state = interactive.then(|| remember(InteractionState::new));
    let ripple_state = interactive.then(|| remember(RippleState::new));
    let checked = controller.with(|c| c.is_checked());
    if interactive {
        modifier = modifier.minimum_interactive_component_size();
        let on_toggle = on_toggle.clone();
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
            Arc::new(move |ctx: PointerEventContext| {
                state.with_mut(|s| s.start_animation_with_spec(ctx.normalized_pos, size, spec));
            })
        });
        let release_handler = ripple_state.map(|state| {
            Arc::new(move |_ctx: PointerEventContext| state.with_mut(|s| s.release()))
        });
        let mut toggle_args = ToggleableArgs::new(
            checked,
            Arc::new(move |_| {
                controller.with_mut(|c| c.toggle());
                let checked = controller.with(|c| c.is_checked());
                if let Some(on_toggle) = on_toggle.as_ref() {
                    on_toggle(checked);
                }
            }),
        )
        .enabled(args.enabled)
        .role(Role::Switch);
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

    let has_thumb_content = child.is_some();
    let progress = controller.with(|c| c.animation_progress());
    let eased_progress = animation::easing(progress);
    let eased_progress_f64 = eased_progress as f64;
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
    let enabled = args.enabled;
    let is_pressed = interaction_state
        .map(|state| state.with(|s| s.is_pressed()))
        .unwrap_or(false);
    let colors = SwitchDefaults::resolve_colors(&args, &scheme, checked, enabled);

    modifier.run(move || {
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
            SurfaceArgs::default()
                .modifier(Modifier::new().size(args.width, args.height))
                .style(track_style)
                .shape(Shape::capsule())
                .show_state_layer(false)
                .show_ripple(false),
            || {},
        );

        // A non-visual state layer for hover + ripple feedback.
        let mut state_layer_args = SurfaceArgs::default()
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
            state_layer_args = state_layer_args.interaction_state(interaction_state);
        }
        let mut state_layer_args = state_layer_args;
        state_layer_args.set_ripple_state(ripple_state);
        surface(state_layer_args, || {});

        let child = child;
        surface(
            SurfaceArgs::default()
                .modifier(Modifier::new().constrain(
                    Some(DimensionValue::Fixed(thumb_size_px)),
                    Some(DimensionValue::Fixed(thumb_size_px)),
                ))
                .style(SurfaceStyle::Filled {
                    color: colors.thumb_color,
                })
                .shape(Shape::Ellipse)
                .content_color(colors.icon_color),
            move || {
                if let Some(child) = child {
                    boxed(
                        BoxedArgs::default()
                            .modifier(Modifier::new().constrain(
                                Some(DimensionValue::Fixed(thumb_size_px)),
                                Some(DimensionValue::Fixed(thumb_size_px)),
                            ))
                            .alignment(Alignment::Center),
                        |scope| {
                            scope.child(move || {
                                child();
                            });
                        },
                    );
                }
            },
        );

        let track_outline_width = args.track_outline_width;
        let track_width = args.width;
        let track_height = args.height;

        layout(SwitchLayout {
            track_width,
            track_height,
            track_outline_width,
            thumb_diameter: thumb_diameter_dp,
            progress: eased_progress_f64,
            checked,
            is_pressed,
        });
    });
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
/// use tessera_ui_basic_components::switch::{SwitchArgs, switch};
///
/// switch(SwitchArgs::default().on_toggle(|checked| {
///     assert!(checked || !checked);
/// }));
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
/// use tessera_ui_basic_components::switch::{SwitchArgs, switch_with_child};
/// use tessera_ui_basic_components::text::{TextArgs, text};
///
/// switch_with_child(
///     SwitchArgs::default().on_toggle(|checked| {
///         assert!(checked || !checked);
///     }),
///     || {
///         text(TextArgs::default().text("✓"));
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
/// use tessera_ui::{remember, tessera};
/// use tessera_ui_basic_components::switch::{
///     SwitchArgs, SwitchController, switch_with_child_and_controller,
/// };
/// use tessera_ui_basic_components::text::{TextArgs, text};
///
/// #[tessera]
/// fn controlled_switch_example() {
///     let controller = remember(|| SwitchController::new(false));
///     switch_with_child_and_controller(
///         SwitchArgs::default().on_toggle(|checked| {
///             println!("Switch is now: {}", checked);
///         }),
///         controller,
///         || {
///             text(TextArgs::default().text("✓"));
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
