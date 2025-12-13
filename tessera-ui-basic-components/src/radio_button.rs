//! Material Design 3 radio button with animated selection feedback.
//! ## Usage Add single-choice selectors to forms, filters, and settings panes.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use closure::closure;
use derive_builder::Builder;
use tessera_ui::{
    Color, DimensionValue, Dp, Px, State,
    accesskit::{Action, Role, Toggled},
    remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    animation,
    boxed::{BoxedArgsBuilder, boxed},
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, SurfaceStyle, surface},
    theme::MaterialColorScheme,
};

const RADIO_ANIMATION_DURATION: Duration = Duration::from_millis(200);
const HOVER_STATE_LAYER_OPACITY: f32 = 0.08;
const RIPPLE_OPACITY: f32 = 0.1;

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
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct RadioButtonArgs {
    /// Callback invoked when the radio transitions to the selected state.
    #[builder(default = "Arc::new(|_| {})")]
    pub on_select: Arc<dyn Fn(bool) + Send + Sync>,
    /// Whether the radio button is currently selected.
    #[builder(default = "false")]
    pub selected: bool,
    /// Visual diameter of the radio glyph (outer ring) in density-independent
    /// pixels.
    #[builder(default = "Dp(20.0)")]
    pub size: Dp,
    /// Minimum interactive touch target for the control.
    #[builder(default = "Dp(48.0)")]
    pub touch_target_size: Dp,
    /// Stroke width applied to the outer ring.
    #[builder(default = "Dp(2.0)")]
    pub stroke_width: Dp,
    /// Diameter of the inner dot when fully selected.
    #[builder(default = "Dp(10.0)")]
    pub dot_size: Dp,
    /// Ring and dot color when selected.
    #[builder(default = "use_context::<MaterialColorScheme>().get().primary")]
    pub selected_color: Color,
    /// Ring color when not selected.
    #[builder(default = "use_context::<MaterialColorScheme>().get().on_surface_variant")]
    pub unselected_color: Color,
    /// Ring and dot color when disabled but selected.
    #[builder(default = "use_context::<MaterialColorScheme>().get().on_surface.with_alpha(0.38)")]
    pub disabled_selected_color: Color,
    /// Ring color when disabled and not selected.
    #[builder(default = "use_context::<MaterialColorScheme>().get().on_surface.with_alpha(0.38)")]
    pub disabled_unselected_color: Color,
    /// Whether the control is interactive.
    #[builder(default = "true")]
    pub enabled: bool,
    /// Optional accessibility label read by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
}

impl Default for RadioButtonArgs {
    fn default() -> Self {
        RadioButtonArgsBuilder::default()
            .build()
            .expect("RadioButtonArgsBuilder default build should succeed")
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
/// use tessera_ui::tessera;
/// use tessera_ui_basic_components::radio_button::{RadioButtonArgsBuilder, radio_button};
///
/// #[tessera]
/// fn radio_demo() {
///     radio_button(
///         RadioButtonArgsBuilder::default()
///             .selected(true)
///             .build()
///             .unwrap(),
///     );
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
    let accessibility_label = args.accessibility_label.clone();
    let accessibility_description = args.accessibility_description.clone();
    let on_select_for_accessibility = args.on_select.clone();
    let enabled_for_accessibility = args.enabled;
    input_handler(Box::new(move |input| {
        controller.with_mut(|c| c.update_animation());
        let selected = controller.with(|c| c.is_selected());

        let mut builder = input.accessibility().role(Role::RadioButton);

        if let Some(label) = accessibility_label.as_ref() {
            builder = builder.label(label.clone());
        }
        if let Some(description) = accessibility_description.as_ref() {
            builder = builder.description(description.clone());
        }

        builder = builder.toggled(if selected {
            Toggled::True
        } else {
            Toggled::False
        });

        if enabled_for_accessibility {
            builder = builder.focusable().action(Action::Click);
        } else {
            builder = builder.disabled();
        }

        builder.commit();

        if enabled_for_accessibility {
            let on_select = on_select_for_accessibility.clone();
            input.set_accessibility_action_handler(move |action| {
                if action == Action::Click && controller.with_mut(|c| c.select()) {
                    on_select(true);
                }
            });
        }
    }));

    controller.with_mut(|c| c.update_animation());
    let progress = controller.with(|c| c.animation_progress());
    let eased_progress = animation::easing(progress);
    let is_selected = controller.with(|c| c.is_selected());

    let target_size = Dp(args.touch_target_size.0.max(args.size.0));
    let padding_dp = Dp(((target_size.0 - args.size.0) / 2.0).max(0.0));

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

    let hover_style = args.enabled.then_some(SurfaceStyle::Filled {
        color: base_state_layer_color.with_alpha(HOVER_STATE_LAYER_OPACITY),
    });

    let ripple_color = if args.enabled {
        base_state_layer_color.with_alpha(RIPPLE_OPACITY)
    } else {
        Color::TRANSPARENT
    };

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

    let on_click = if args.enabled {
        Some(
            Arc::new(closure!(clone args.on_select, clone controller, || {
                if controller.with_mut(|c| c.select()) {
                    on_select(true);
                }
            })) as Arc<dyn Fn() + Send + Sync>,
        )
    } else {
        None
    };

    let mut root_builder = SurfaceArgsBuilder::default()
        .width(DimensionValue::Fixed(target_size.to_px()))
        .height(DimensionValue::Fixed(target_size.to_px()))
        .padding(padding_dp)
        .shape(Shape::Ellipse)
        .style(SurfaceStyle::Filled {
            color: Color::TRANSPARENT,
        })
        .hover_style(hover_style)
        .ripple_color(ripple_color)
        .accessibility_role(Role::RadioButton);

    if let Some(on_click) = on_click.clone() {
        root_builder = root_builder.on_click_shared(on_click);
    }

    surface(
        root_builder.build().expect("builder construction failed"),
        {
            let args = args.clone();
            move || {
                surface(
                    SurfaceArgsBuilder::default()
                        .width(DimensionValue::Fixed(args.size.to_px()))
                        .height(DimensionValue::Fixed(args.size.to_px()))
                        .shape(Shape::Ellipse)
                        .style(ring_style)
                        .build()
                        .expect("builder construction failed"),
                    {
                        let dot_size_px = args.dot_size.to_px();
                        move || {
                            let animated_size =
                                (dot_size_px.0 as f32 * eased_progress).round() as i32;
                            if animated_size > 0 {
                                boxed(
                                    BoxedArgsBuilder::default()
                                        .alignment(Alignment::Center)
                                        .width(DimensionValue::Fixed(args.size.to_px()))
                                        .height(DimensionValue::Fixed(args.size.to_px()))
                                        .build()
                                        .expect("builder construction failed"),
                                    |scope| {
                                        scope.child({
                                            let dot_color = active_dot_color;
                                            move || {
                                                surface(
                                                    SurfaceArgsBuilder::default()
                                                        .width(DimensionValue::Fixed(Px(
                                                            animated_size,
                                                        )))
                                                        .height(DimensionValue::Fixed(Px(
                                                            animated_size,
                                                        )))
                                                        .shape(Shape::Ellipse)
                                                        .style(SurfaceStyle::Filled {
                                                            color: dot_color,
                                                        })
                                                        .build()
                                                        .expect("builder construction failed"),
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
            }
        },
    );
}
