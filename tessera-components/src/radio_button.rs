//! Material Design 3 radio button with animated selection feedback.
//!
//! ## Usage
//!
//! Add single-choice selectors to forms, filters, and settings panes.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use closure::closure;
use derive_setters::Setters;
use tessera_ui::{
    Color, DimensionValue, Dp, Modifier, Px, PxSize, State, accesskit::Role, remember, tessera,
    use_context,
};

use crate::{
    alignment::Alignment,
    animation,
    boxed::{BoxedArgs, boxed},
    modifier::{InteractionState, ModifierExt as _, PointerEventContext, SelectableArgs},
    ripple_state::{RippleSpec, RippleState},
    shape_def::Shape,
    surface::{SurfaceArgs, SurfaceStyle, surface},
    theme::{MaterialAlpha, MaterialTheme},
};

const RADIO_ANIMATION_DURATION: Duration = Duration::from_millis(200);

/// Material Design 3 defaults for [`radio_button`].
pub struct RadioButtonDefaults;

impl RadioButtonDefaults {
    /// State-layer size used for hover/press feedback.
    pub const STATE_LAYER_SIZE: Dp = Dp(40.0);
}

/// Shared state for the `radio_button` component, including selection
/// animation.
#[derive(Clone)]
pub struct RadioButtonController {
    selected: bool,
    progress: f32,
    start_progress: f32,
    last_change_time: Option<Instant>,
}

impl Default for RadioButtonController {
    fn default() -> Self {
        Self::new(false)
    }
}

impl RadioButtonController {
    /// Creates a new radio button state with the given initial selection.
    pub fn new(selected: bool) -> Self {
        let progress = if selected { 1.0 } else { 0.0 };
        Self {
            selected,
            progress,
            start_progress: progress,
            last_change_time: None,
        }
    }

    /// Returns whether the radio button is currently selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// Sets the selection state, starting an animation when the value changes.
    pub fn set_selected(&mut self, selected: bool) {
        if self.selected != selected {
            self.selected = selected;
            self.start_progress = self.progress;
            self.last_change_time = Some(Instant::now());
        }
    }

    /// Marks the radio button as selected, returning `true` if this triggered a
    /// state change.
    pub fn select(&mut self) -> bool {
        if self.selected {
            return false;
        }
        self.selected = true;
        self.start_progress = self.progress;
        self.last_change_time = Some(Instant::now());
        true
    }

    fn update_animation(&mut self) {
        if let Some(start) = self.last_change_time {
            let elapsed = start.elapsed();
            let fraction =
                (elapsed.as_secs_f32() / RADIO_ANIMATION_DURATION.as_secs_f32()).min(1.0);
            let target = if self.selected { 1.0 } else { 0.0 };
            self.progress = self.start_progress + (target - self.start_progress) * fraction;
            if fraction >= 1.0 {
                self.last_change_time = None;
                self.progress = target;
                self.start_progress = target;
            }
        }
    }

    fn animation_progress(&self) -> f32 {
        self.progress
    }
}

/// Arguments for configuring the `radio_button` component.
#[derive(Clone, Setters)]
pub struct RadioButtonArgs {
    /// Optional modifier chain applied to the radio button subtree.
    pub modifier: Modifier,
    /// Callback invoked when the radio transitions to the selected state.
    #[setters(skip)]
    pub on_select: Arc<dyn Fn(bool) + Send + Sync>,
    /// Whether the radio button is currently selected.
    pub selected: bool,
    /// Visual diameter of the radio glyph (outer ring) in density-independent
    /// pixels.
    pub size: Dp,
    /// Minimum interactive touch target for the control.
    pub touch_target_size: Dp,
    /// Stroke width applied to the outer ring.
    pub stroke_width: Dp,
    /// Diameter of the inner dot when fully selected.
    pub dot_size: Dp,
    /// Ring and dot color when selected.
    pub selected_color: Color,
    /// Ring color when not selected.
    pub unselected_color: Color,
    /// Ring and dot color when disabled but selected.
    pub disabled_selected_color: Color,
    /// Ring color when disabled and not selected.
    pub disabled_unselected_color: Color,
    /// Whether the control is interactive.
    pub enabled: bool,
    /// Optional accessibility label read by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
}

impl RadioButtonArgs {
    /// Sets the on_select handler.
    pub fn on_select<F>(mut self, on_select: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_select = Arc::new(on_select);
        self
    }

    /// Sets the on_select handler using a shared callback.
    pub fn on_select_shared(mut self, on_select: Arc<dyn Fn(bool) + Send + Sync>) -> Self {
        self.on_select = on_select;
        self
    }
}

impl Default for RadioButtonArgs {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        Self {
            modifier: Modifier::new(),
            on_select: Arc::new(|_| {}),
            selected: false,
            size: Dp(20.0),
            touch_target_size: Dp(48.0),
            stroke_width: Dp(2.0),
            dot_size: Dp(10.0),
            selected_color: scheme.primary,
            unselected_color: scheme.on_surface_variant,
            disabled_selected_color: scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTENT),
            disabled_unselected_color: scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTENT),
            enabled: true,
            accessibility_label: None,
            accessibility_description: None,
        }
    }
}

fn interpolate_color(a: Color, b: Color, t: f32) -> Color {
    let factor = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * factor,
        g: a.g + (b.g - a.g) * factor,
        b: a.b + (b.b - a.b) * factor,
        a: a.a + (b.a - a.a) * factor,
    }
}

/// # radio_button
///
/// Render a Material Design 3 radio button with a smooth animated selection
/// dot.
///
/// ## Usage
///
/// Use in single-choice groups where exactly one option should be active.
///
/// ## Parameters
///
/// - `args` — configures sizing, colors, and callbacks; see
///   [`RadioButtonArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_components::radio_button::{RadioButtonArgs, radio_button};
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn radio_demo() {
///     radio_button(RadioButtonArgs::default().selected(true));
/// }
/// ```
#[tessera]
pub fn radio_button(args: impl Into<RadioButtonArgs>) {
    let args: RadioButtonArgs = args.into();
    let controller = remember(|| RadioButtonController::new(args.selected));

    if controller.with(|c| c.is_selected()) != args.selected {
        controller.with_mut(|c| c.set_selected(args.selected));
    }

    radio_button_with_controller(args, controller);
}

/// # radio_button_with_controller
///
/// Render a Material Design 3 radio button with an external controller.
///
/// ## Parameters
///
/// - `args` — configures sizing, colors, and callbacks; see
///   [`RadioButtonArgs`].
/// - `controller` — a clonable [`RadioButtonController`] that manages selection
///   animation.
#[tessera]
pub fn radio_button_with_controller(
    args: impl Into<RadioButtonArgs>,
    controller: State<RadioButtonController>,
) {
    let args: RadioButtonArgs = args.into();
    controller.with_mut(|c| c.update_animation());
    let progress = controller.with(|c| c.animation_progress());
    let eased_progress = animation::easing(progress);
    let is_selected = controller.with(|c| c.is_selected());
    let interaction_state = args.enabled.then(|| remember(InteractionState::new));
    let ripple_state = args.enabled.then(|| remember(RippleState::new));

    let target_size = Dp(args.touch_target_size.0.max(args.size.0));

    let ring_color = if args.enabled {
        interpolate_color(args.unselected_color, args.selected_color, progress)
    } else if is_selected {
        args.disabled_selected_color
    } else {
        args.disabled_unselected_color
    };

    let base_state_layer_color = if args.enabled {
        ring_color
    } else if is_selected {
        args.disabled_selected_color
    } else {
        args.disabled_unselected_color
    };

    let ripple_color = base_state_layer_color;

    let target_dot_color = if args.enabled {
        args.selected_color
    } else {
        args.disabled_selected_color
    };
    let active_dot_color = interpolate_color(Color::TRANSPARENT, target_dot_color, eased_progress);

    let ring_style = SurfaceStyle::Outlined {
        color: ring_color,
        width: args.stroke_width,
    };

    let on_click = Arc::new(closure!(clone args.on_select, clone controller, || {
        if controller.with_mut(|c| c.select()) {
            on_select(true);
        }
    })) as Arc<dyn Fn() + Send + Sync>;

    let state_layer_size = RadioButtonDefaults::STATE_LAYER_SIZE;
    let state_layer_radius = Dp(state_layer_size.0 / 2.0);

    let mut state_layer_args = SurfaceArgs::default()
        .modifier(Modifier::new().size(state_layer_size, state_layer_size))
        .shape(Shape::Ellipse)
        .enabled(args.enabled)
        .style(SurfaceStyle::Filled {
            color: Color::TRANSPARENT,
        })
        .ripple_bounded(false)
        .ripple_radius(state_layer_radius)
        .ripple_color(ripple_color);

    if let Some(state) = interaction_state {
        state_layer_args = state_layer_args.interaction_state(state);
    }

    let mut state_layer_args = state_layer_args;
    state_layer_args.set_ripple_state(ripple_state);

    let mut modifier = args.modifier.size(target_size, target_size);
    if args.enabled {
        let ripple_spec = RippleSpec {
            bounded: false,
            radius: Some(state_layer_radius),
        };
        let ripple_size = PxSize::new(state_layer_size.to_px(), state_layer_size.to_px());
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
        let mut selectable_args = SelectableArgs::new(is_selected, on_click.clone())
            .enabled(true)
            .role(Role::RadioButton);
        if let Some(label) = args.accessibility_label.clone() {
            selectable_args = selectable_args.label(label);
        }
        if let Some(desc) = args.accessibility_description.clone() {
            selectable_args = selectable_args.description(desc);
        }
        if let Some(state) = interaction_state {
            selectable_args = selectable_args.interaction_state(state);
        }
        if let Some(handler) = press_handler {
            selectable_args = selectable_args.on_press(handler);
        }
        if let Some(handler) = release_handler {
            selectable_args = selectable_args.on_release(handler);
        }
        modifier = modifier.selectable(selectable_args);
    }

    boxed(
        BoxedArgs::default()
            .modifier(modifier)
            .alignment(Alignment::Center),
        move |scope| {
            let args = args.clone();
            let ring_style = ring_style.clone();
            scope.child(move || {
                surface(state_layer_args, move || {
                    boxed(
                        BoxedArgs::default()
                            .alignment(Alignment::Center)
                            .modifier(Modifier::new().fill_max_size()),
                        move |center| {
                            let args = args.clone();
                            let ring_style = ring_style.clone();
                            center.child(move || {
                                surface(
                                    SurfaceArgs::default()
                                        .modifier(Modifier::new().size(args.size, args.size))
                                        .shape(Shape::Ellipse)
                                        .style(ring_style),
                                    {
                                        let dot_size_px = args.dot_size.to_px();
                                        move || {
                                            let animated_size =
                                                (dot_size_px.0 as f32 * eased_progress).round()
                                                    as i32;
                                            if animated_size > 0 {
                                                boxed(
                                                    BoxedArgs::default()
                                                        .alignment(Alignment::Center)
                                                        .modifier(Modifier::new().size(
                                                            args.size,
                                                            args.size,
                                                        )),
                                                    |dot_scope| {
                                                        dot_scope.child({
                                                            let dot_color = active_dot_color;
                                                            move || {
                                                                surface(
                                                                    SurfaceArgs::default()
                                                                        .modifier(
                                                                            Modifier::new().constrain(
                                                                                Some(
                                                                                    DimensionValue::Fixed(
                                                                                        Px(animated_size),
                                                                                    ),
                                                                                ),
                                                                                Some(
                                                                                    DimensionValue::Fixed(
                                                                                        Px(animated_size),
                                                                                    ),
                                                                                ),
                                                                            ),
                                                                        )
                                                                        .shape(Shape::Ellipse)
                                                                        .style(
                                                                            SurfaceStyle::Filled {
                                                                                color: dot_color,
                                                                            },
                                                                        ),
                                                                    || {},
                                                                );
                                                            }
                                                        });
                                                    },
                                                );
                                            }
                                        }
                                    },
                                );
                            });
                        },
                    );
                });
            });
        },
    );
}
