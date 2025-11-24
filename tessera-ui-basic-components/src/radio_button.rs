//! Material Design 3 radio button with animated selection feedback.
//! ## Usage Add single-choice selectors to forms, filters, and settings panes.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{
    Color, DimensionValue, Dp, Px,
    accesskit::{Action, Role, Toggled},
    tessera,
};

use crate::{
    RippleState,
    alignment::Alignment,
    animation,
    boxed::{BoxedArgsBuilder, boxed},
    material_color,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, SurfaceStyle, surface},
};

const RADIO_ANIMATION_DURATION: Duration = Duration::from_millis(200);
const HOVER_STATE_LAYER_OPACITY: f32 = 0.08;
const RIPPLE_OPACITY: f32 = 0.12;

/// Shared state for the `radio_button` component, including ripple feedback and selection animation.
#[derive(Clone)]
pub struct RadioButtonState {
    ripple: RippleState,
    selection: Arc<RwLock<RadioSelectionState>>,
}

impl Default for RadioButtonState {
    fn default() -> Self {
        Self::new(false)
    }
}

impl RadioButtonState {
    /// Creates a new radio button state with the given initial selection.
    pub fn new(selected: bool) -> Self {
        Self {
            ripple: RippleState::new(),
            selection: Arc::new(RwLock::new(RadioSelectionState::new(selected))),
        }
    }

    /// Returns whether the radio button is currently selected.
    pub fn is_selected(&self) -> bool {
        self.selection.read().selected
    }

    /// Sets the selection state, starting an animation when the value changes.
    pub fn set_selected(&self, selected: bool) {
        let mut selection = self.selection.write();
        if selection.selected != selected {
            selection.selected = selected;
            selection.start_progress = selection.progress;
            selection.last_change_time = Some(Instant::now());
        }
    }

    /// Marks the radio button as selected, returning `true` if this triggered a state change.
    pub fn select(&self) -> bool {
        let mut selection = self.selection.write();
        if selection.selected {
            return false;
        }
        selection.selected = true;
        selection.start_progress = selection.progress;
        selection.last_change_time = Some(Instant::now());
        true
    }

    fn update_animation(&self) {
        let mut selection = self.selection.write();
        if let Some(start) = selection.last_change_time {
            let elapsed = start.elapsed();
            let fraction =
                (elapsed.as_secs_f32() / RADIO_ANIMATION_DURATION.as_secs_f32()).min(1.0);
            let target = if selection.selected { 1.0 } else { 0.0 };
            selection.progress =
                selection.start_progress + (target - selection.start_progress) * fraction;
            if fraction >= 1.0 {
                selection.last_change_time = None;
                selection.progress = target;
                selection.start_progress = target;
            }
        }
    }

    fn animation_progress(&self) -> f32 {
        self.selection.read().progress
    }

    fn ripple_state(&self) -> RippleState {
        self.ripple.clone()
    }
}

struct RadioSelectionState {
    selected: bool,
    progress: f32,
    start_progress: f32,
    last_change_time: Option<Instant>,
}

impl RadioSelectionState {
    fn new(selected: bool) -> Self {
        let progress = if selected { 1.0 } else { 0.0 };
        Self {
            selected,
            progress,
            start_progress: progress,
            last_change_time: None,
        }
    }
}

/// Arguments for configuring the `radio_button` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct RadioButtonArgs {
    /// Callback invoked when the radio transitions to the selected state.
    #[builder(default = "Arc::new(|_| {})")]
    pub on_select: Arc<dyn Fn(bool) + Send + Sync>,
    /// Visual diameter of the radio glyph (outer ring) in density-independent pixels.
    #[builder(default = "Dp(18.0)")]
    pub size: Dp,
    /// Minimum interactive touch target for the control.
    #[builder(default = "Dp(30.0)")]
    pub touch_target_size: Dp,
    /// Stroke width applied to the outer ring.
    #[builder(default = "Dp(1.5)")]
    pub stroke_width: Dp,
    /// Diameter of the inner dot when fully selected.
    #[builder(default = "Dp(9.0)")]
    pub dot_size: Dp,
    /// Ring and dot color when selected.
    #[builder(default = "material_color::global_material_scheme().primary")]
    pub selected_color: Color,
    /// Ring color when not selected.
    #[builder(default = "material_color::global_material_scheme().on_surface_variant")]
    pub unselected_color: Color,
    /// Ring and dot color when disabled but selected.
    #[builder(default = "material_color::global_material_scheme().on_surface.with_alpha(0.38)")]
    pub disabled_selected_color: Color,
    /// Ring color when disabled and not selected.
    #[builder(default = "material_color::global_material_scheme().on_surface.with_alpha(0.38)")]
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
/// Render a Material Design 3 radio button with a smooth animated selection dot.
///
/// ## Usage
///
/// Use in single-choice groups where exactly one option should be active.
///
/// ## Parameters
///
/// - `args` — configures sizing, colors, and callbacks; see [`RadioButtonArgs`].
/// - `state` — a clonable [`RadioButtonState`] that manages selection animation and ripple feedback.
///
/// ## Examples
///
/// ```
/// use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
/// use tessera_ui::tessera;
/// use tessera_ui_basic_components::radio_button::{radio_button, RadioButtonArgsBuilder, RadioButtonState};
///
/// #[derive(Clone, Default)]
/// struct DemoState {
///     selected: Arc<AtomicBool>,
///     radio: RadioButtonState,
/// }
///
/// #[tessera]
/// fn radio_demo(state: DemoState) {
///     let on_select = Arc::new({
///         let selected = state.selected.clone();
///         move |is_selected| {
///             selected.store(is_selected, Ordering::SeqCst);
///         }
///     });
///
///     radio_button(
///         RadioButtonArgsBuilder::default()
///             .on_select(on_select)
///             .build()
///             .unwrap(),
///         state.radio.clone(),
///     );
///
///     state.radio.set_selected(true);
///     assert!(state.radio.is_selected());
///     state.radio.set_selected(false);
///     assert!(!state.radio.is_selected());
/// }
/// ```
#[tessera]
pub fn radio_button(args: impl Into<RadioButtonArgs>, state: RadioButtonState) {
    let args: RadioButtonArgs = args.into();

    let state_for_accessibility = state.clone();
    let state_for_animation = state.clone();
    let accessibility_label = args.accessibility_label.clone();
    let accessibility_description = args.accessibility_description.clone();
    let on_select_for_accessibility = args.on_select.clone();
    let enabled_for_accessibility = args.enabled;
    input_handler(Box::new(move |input| {
        state_for_animation.update_animation();
        let selected = state_for_animation.is_selected();

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
            let state = state_for_accessibility.clone();
            let on_select = on_select_for_accessibility.clone();
            input.set_accessibility_action_handler(move |action| {
                if action == Action::Click && state.select() {
                    on_select(true);
                }
            });
        }
    }));

    state.update_animation();
    let progress = state.animation_progress();
    let eased_progress = animation::easing(progress);
    let is_selected = state.is_selected();

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
        let on_select = args.on_select.clone();
        let state_for_click = state.clone();
        Some(Arc::new(move || {
            if state_for_click.select() {
                on_select(true);
            }
        }) as Arc<dyn Fn() + Send + Sync>)
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
        root_builder = root_builder.on_click(on_click);
    }

    surface(
        root_builder.build().expect("builder construction failed"),
        args.enabled.then(|| state.ripple_state()),
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
                    None,
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
                                                    None,
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
