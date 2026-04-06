//! Material Design 3 radio button with animated selection feedback.
//!
//! ## Usage
//!
//! Add single-choice selectors to forms, filters, and settings panes.

use std::time::Duration;

use tessera_ui::{
    AxisConstraint, Callback, CallbackWith, Color, Dp, FocusState, FocusTraversalPolicy, Modifier,
    Px, PxSize, RenderSlot, State, accesskit::Role, current_frame_nanos, layout::layout,
    modifier::FocusModifierExt as _, provide_context, receive_frame_nanos, remember, tessera,
    use_context,
};

use crate::{
    alignment::Alignment,
    animation,
    boxed::boxed,
    modifier::{InteractionState, ModifierExt as _, PointerEventContext, SelectableArgs},
    ripple_state::{RippleSpec, RippleState},
    shape_def::Shape,
    surface::{SurfaceStyle, surface},
    theme::{MaterialAlpha, MaterialTheme},
};

const RADIO_ANIMATION_DURATION: Duration = Duration::from_millis(200);

#[derive(Clone, Copy, Debug)]
struct RadioGroupContext;

/// Orientation for [`radio_group`] traversal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadioGroupOrientation {
    /// Traverse radios vertically with `Up` and `Down`.
    #[default]
    Vertical,
    /// Traverse radios horizontally with `Left` and `Right`.
    Horizontal,
}

/// Material Design 3 defaults for [`radio_button`].
pub struct RadioButtonDefaults;

impl RadioButtonDefaults {
    /// State-layer size used for hover/press feedback.
    pub const STATE_LAYER_SIZE: Dp = Dp(40.0);
}

/// Shared state for the `radio_button` component, including selection
/// animation.
#[derive(Clone, PartialEq)]
pub struct RadioButtonController {
    selected: bool,
    progress: f32,
    start_progress: f32,
    last_change_frame_nanos: Option<u64>,
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
            last_change_frame_nanos: None,
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
            self.last_change_frame_nanos = Some(current_frame_nanos());
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
        self.last_change_frame_nanos = Some(current_frame_nanos());
        true
    }

    fn update_animation(&mut self, frame_nanos: u64) {
        if let Some(start_frame_nanos) = self.last_change_frame_nanos {
            let elapsed_nanos = frame_nanos.saturating_sub(start_frame_nanos);
            let animation_nanos = RADIO_ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
            let fraction = if animation_nanos == 0 {
                1.0
            } else {
                (elapsed_nanos as f32 / animation_nanos as f32).min(1.0)
            };
            let target = if self.selected { 1.0 } else { 0.0 };
            self.progress = self.start_progress + (target - self.start_progress) * fraction;
            if fraction >= 1.0 {
                self.last_change_frame_nanos = None;
                self.progress = target;
                self.start_progress = target;
            }
        }
    }

    fn animation_progress(&self) -> f32 {
        self.progress
    }

    fn is_animating(&self) -> bool {
        self.last_change_frame_nanos.is_some()
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

/// # radio_group
///
/// Provide roving-focus keyboard navigation for a single-choice group of radio
/// buttons.
///
/// ## Usage
///
/// Use around related radio buttons so arrow keys move within the group and
/// focus change selects the active radio.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the group container.
/// - `orientation` — optional traversal direction.
/// - `wrap` — whether traversal wraps when it reaches either end.
/// - `content` — optional group content slot.
///
/// ## Examples
///
/// ```rust
/// use tessera_components::radio_button::{radio_button, radio_group};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn radio_group_demo() {
///     let selected = remember(|| 0usize);
///     radio_group().content(move || {
///         radio_button()
///             .selected(selected.get() == 0)
///             .on_select({ move |_| selected.set(0) });
///         radio_button()
///             .selected(selected.get() == 1)
///             .on_select({ move |_| selected.set(1) });
///     });
/// }
/// ```
#[tessera]
pub fn radio_group(
    modifier: Modifier,
    orientation: Option<RadioGroupOrientation>,
    wrap: bool,
    content: Option<RenderSlot>,
) {
    let content = content.unwrap_or_else(RenderSlot::empty);
    let modifier = modifier.focus_group().focus_traversal_policy(
        match orientation.unwrap_or_default() {
            RadioGroupOrientation::Horizontal => FocusTraversalPolicy::horizontal(),
            RadioGroupOrientation::Vertical => FocusTraversalPolicy::vertical(),
        }
        .wrap(wrap),
    );
    layout().modifier(modifier).child(move || {
        let content = content;
        provide_context(
            || RadioGroupContext,
            move || {
                content.render();
            },
        );
    });
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
/// - `modifier` — optional modifier chain applied to the radio button subtree.
/// - `on_select` — optional callback invoked when the radio transitions to the
///   selected state.
/// - `selected` — whether the radio button is currently selected.
/// - `size` — optional visual diameter of the radio glyph.
/// - `touch_target_size` — optional minimum interactive touch target.
/// - `stroke_width` — optional stroke width applied to the outer ring.
/// - `dot_size` — optional diameter of the inner dot when fully selected.
/// - `selected_color` — optional ring and dot color when selected.
/// - `unselected_color` — optional ring color when not selected.
/// - `disabled_selected_color` — optional ring and dot color when disabled but
///   selected.
/// - `disabled_unselected_color` — optional ring color when disabled and not
///   selected.
/// - `enabled` — whether the control is interactive.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `controller` — optional external controller for selection state and
///   animation.
///
/// ## Examples
///
/// ```rust
/// use tessera_components::radio_button::radio_button;
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn radio_demo() {
///     radio_button().selected(true);
/// }
/// ```
#[tessera]
pub fn radio_button(
    modifier: Modifier,
    on_select: Option<CallbackWith<bool, ()>>,
    selected: bool,
    size: Option<Dp>,
    touch_target_size: Option<Dp>,
    stroke_width: Option<Dp>,
    dot_size: Option<Dp>,
    selected_color: Option<Color>,
    unselected_color: Option<Color>,
    disabled_selected_color: Option<Color>,
    disabled_unselected_color: Option<Color>,
    enabled: bool,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    controller: Option<State<RadioButtonController>>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let size = size.unwrap_or(Dp(20.0));
    let touch_target_size = touch_target_size.unwrap_or(Dp(48.0));
    let stroke_width = stroke_width.unwrap_or(Dp(2.0));
    let dot_size = dot_size.unwrap_or(Dp(10.0));
    let selected_color = selected_color.unwrap_or(scheme.primary);
    let unselected_color = unselected_color.unwrap_or(scheme.on_surface_variant);
    let disabled_selected_color = disabled_selected_color.unwrap_or(
        scheme
            .on_surface
            .with_alpha(MaterialAlpha::DISABLED_CONTENT),
    );
    let disabled_unselected_color = disabled_unselected_color.unwrap_or(
        scheme
            .on_surface
            .with_alpha(MaterialAlpha::DISABLED_CONTENT),
    );
    let on_select = on_select.unwrap_or_else(CallbackWith::default_value);

    let controller =
        controller.unwrap_or_else(|| remember(|| RadioButtonController::new(selected)));

    if controller.with(|c| c.is_selected()) != selected {
        controller.with_mut(|c| c.set_selected(selected));
    }

    let mut builder = radio_button_inner()
        .modifier(modifier)
        .on_select_shared(on_select)
        .selected(selected)
        .size(size)
        .touch_target_size(touch_target_size)
        .stroke_width(stroke_width)
        .dot_size(dot_size)
        .selected_color(selected_color)
        .unselected_color(unselected_color)
        .disabled_selected_color(disabled_selected_color)
        .disabled_unselected_color(disabled_unselected_color)
        .enabled(enabled)
        .controller(controller);
    if let Some(label) = accessibility_label {
        builder = builder.accessibility_label(label);
    }
    if let Some(description) = accessibility_description {
        builder = builder.accessibility_description(description);
    }
    drop(builder);
}

#[tessera]
fn radio_button_inner(
    modifier: Modifier,
    on_select: Option<CallbackWith<bool, ()>>,
    selected: bool,
    size: Dp,
    touch_target_size: Dp,
    stroke_width: Dp,
    dot_size: Dp,
    selected_color: Color,
    unselected_color: Color,
    disabled_selected_color: Color,
    disabled_unselected_color: Color,
    enabled: bool,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    controller: Option<State<RadioButtonController>>,
) {
    let _ = selected;
    let radio_group = use_context::<RadioGroupContext>().map(|context| context.get());
    let controller = controller.expect("radio_button_inner requires controller to be set");
    if controller.with(|c| c.is_animating()) {
        receive_frame_nanos(move |frame_nanos| {
            let is_animating = controller.with_mut(|controller| {
                controller.update_animation(frame_nanos);
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
    let eased_progress = animation::easing(progress);
    let is_selected = controller.with(|c| c.is_selected());
    let interaction_state = enabled.then(|| remember(InteractionState::new));
    let ripple_state = enabled.then(|| remember(RippleState::new));
    let on_select = on_select.unwrap_or_else(CallbackWith::default_value);

    let target_size = Dp(touch_target_size.0.max(size.0));

    let ring_color = if enabled {
        interpolate_color(unselected_color, selected_color, progress)
    } else if is_selected {
        disabled_selected_color
    } else {
        disabled_unselected_color
    };

    let base_state_layer_color = if enabled {
        ring_color
    } else if is_selected {
        disabled_selected_color
    } else {
        disabled_unselected_color
    };

    let ripple_color = base_state_layer_color;

    let target_dot_color = if enabled {
        selected_color
    } else {
        disabled_selected_color
    };
    let active_dot_color = interpolate_color(Color::TRANSPARENT, target_dot_color, eased_progress);

    let on_click = move || {
        if controller.with_mut(|c| c.select()) {
            on_select.call(true);
        }
    };

    let state_layer_size = RadioButtonDefaults::STATE_LAYER_SIZE;
    let state_layer_radius = Dp(state_layer_size.0 / 2.0);

    let mut modifier = modifier.size(target_size, target_size);
    if enabled && radio_group.is_some() {
        modifier = modifier.on_focus_changed(move |focus_state: FocusState| {
            if focus_state.has_focus() && controller.with_mut(|controller| controller.select()) {
                on_select.call(true);
            }
        });
    }

    if enabled {
        let ripple_spec = RippleSpec {
            bounded: false,
            radius: Some(state_layer_radius),
        };
        let ripple_size = PxSize::new(state_layer_size.to_px(), state_layer_size.to_px());
        let press_handler = ripple_state.map(|state| {
            let spec = ripple_spec;
            let size = ripple_size;
            move |ctx: PointerEventContext| {
                state.with_mut(|s| s.start_animation_with_spec(ctx.normalized_pos, size, spec));
            }
        });
        let release_handler = ripple_state
            .map(|state| move |_ctx: PointerEventContext| state.with_mut(|s| s.release()));
        let selectable_args = SelectableArgs {
            selected: is_selected,
            on_click: Callback::new(on_click),
            enabled: true,
            role: Some(Role::RadioButton),
            label: accessibility_label.clone(),
            description: accessibility_description.clone(),
            interaction_state,
            on_press: press_handler.map(Into::into),
            on_release: release_handler.map(Into::into),
            ..Default::default()
        };
        modifier = modifier.selectable_with(selectable_args);
    }

    boxed()
        .modifier(modifier)
        .alignment(Alignment::Center)
        .children(move || {
            let interaction_state = interaction_state;
            let ripple_state = ripple_state;
            let mut builder = surface()
                .modifier(Modifier::new().size(state_layer_size, state_layer_size))
                .shape(Shape::Ellipse)
                .enabled(enabled)
                .style(SurfaceStyle::Filled {
                    color: Color::TRANSPARENT,
                })
                .ripple_bounded(false)
                .ripple_radius(state_layer_radius)
                .ripple_color(ripple_color);
            if let Some(state) = interaction_state {
                builder = builder.interaction_state(state);
            }
            builder.set_ripple_state(ripple_state);
            builder.with_child({
                move || {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            surface()
                                .modifier(Modifier::new().size(size, size))
                                .shape(Shape::Ellipse)
                                .style(SurfaceStyle::Outlined {
                                    color: ring_color,
                                    width: stroke_width,
                                })
                                .with_child({
                                    move || {
                                        let animated_size =
                                            (dot_size.to_px().0 as f32 * eased_progress).round()
                                                as i32;
                                        if animated_size > 0 {
                                            boxed()
                                                .alignment(Alignment::Center)
                                                .modifier(Modifier::new().size(size, size))
                                                .children(move || {
                                                    surface()
                                                        .modifier(Modifier::new().constrain(
                                                            Some(AxisConstraint::exact(Px(
                                                                animated_size,
                                                            ))),
                                                            Some(AxisConstraint::exact(Px(
                                                                animated_size,
                                                            ))),
                                                        ))
                                                        .shape(Shape::Ellipse)
                                                        .style(SurfaceStyle::Filled {
                                                            color: active_dot_color,
                                                        })
                                                        .with_child(|| {});
                                                });
                                        }
                                    }
                                });
                        });
                }
            });
        });
}
