//! Material Design 3 radio button with animated selection feedback.
//!
//! ## Usage
//!
//! Add single-choice selectors to forms, filters, and settings panes.

use std::time::Duration;

use tessera_foundation::gesture::TapRecognizer;
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, Callback, CallbackWith, Color,
    Dp, FocusDirection, FocusRequester, FocusState, FocusTraversalPolicy, Modifier, Px, PxSize,
    RenderSlotWith, SemanticsModifierNode, State,
    accesskit::{Orientation, Role, Toggled},
    current_frame_nanos, focus,
    layout::Stack,
    layout::layout,
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
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

#[cfg(all(test, not(target_family = "wasm")))]
mod tests;

struct RadioSemantics {
    group: bool,
    selected: bool,
    label: Option<String>,
    description: Option<String>,
    orientation: Option<Orientation>,
}

impl SemanticsModifierNode for RadioSemantics {
    fn apply(
        &self,
        node: &mut AccessibilityNode,
        handler: &mut Option<AccessibilityActionHandler>,
    ) {
        node.role = Some(if self.group {
            Role::RadioGroup
        } else {
            Role::RadioButton
        });
        if let Some(label) = &self.label {
            node.label = Some(label.clone());
        }
        if let Some(description) = &self.description {
            node.description = Some(description.clone());
        }
        if let Some(orientation) = self.orientation {
            node.orientation = Some(orientation);
        }
        if !self.group {
            node.toggled = Some(if self.selected {
                Toggled::True
            } else {
                Toggled::False
            });
            node.disabled = true;
            node.focusable = false;
            node.actions.clear();
            *handler = None;
        }
    }
}

/// Marks the choices of a radio group for its keyboard navigation.
#[derive(Clone, Copy, PartialEq)]
pub struct RadioGroupScope(());

/// Physical orientation announced for [`radio_group`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadioGroupOrientation {
    /// Announce a vertical group; all four arrows navigate the choices.
    #[default]
    Vertical,
    /// Announce a horizontal group; all four arrows navigate the choices.
    Horizontal,
}

/// Material Design 3 defaults for [`radio_button`].
pub struct RadioButtonDefaults;

impl RadioButtonDefaults {
    /// State-layer size used for hover/press feedback.
    pub const STATE_LAYER_SIZE: Dp = Dp(40.0);
}

#[derive(Clone, PartialEq)]
struct RadioButtonMotion {
    selected: bool,
    progress: f32,
    start_progress: f32,
    last_change_frame_nanos: Option<u64>,
}

impl RadioButtonMotion {
    fn new(selected: bool) -> Self {
        let progress = if selected { 1.0 } else { 0.0 };
        Self {
            selected,
            progress,
            start_progress: progress,
            last_change_frame_nanos: None,
        }
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        if self.selected != selected {
            self.selected = selected;
            self.start_progress = self.progress;
            self.last_change_frame_nanos = Some(current_frame_nanos());
        }
    }

    fn update_animation(&mut self, frame_nanos: u64) {
        if let Some(start_frame_nanos) = self.last_change_frame_nanos {
            let elapsed_nanos = frame_nanos.saturating_sub(start_frame_nanos);
            let animation_nanos = RADIO_ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
            let fraction = (elapsed_nanos as f32 / animation_nanos as f32).min(1.0);
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
/// Provide directional keyboard navigation for a single-choice group of radio
/// buttons.
///
/// ## Usage
///
/// Use around related radio buttons so arrow keys move and select within the
/// group while pointer selection waits for a completed click. Tab enters at the
/// selected enabled choice, or the first available choice, and then leaves the
/// whole group.
/// The caller owns the selection: derive every choice's `selected` value from
/// the same application state and update it from `on_select`.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the group container.
/// - `orientation` — physical direction announced to assistive technology;
///   defaults to vertical. All four arrows navigate in declaration order.
/// - `wrap` — whether arrow traversal wraps at either end; defaults to `true`.
/// - `accessibility_label` — optional name for the related choices.
/// - `accessibility_description` — optional additional group instructions.
/// - `content` — content receiving the group scope for its choices.
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
///     radio_group().content(move |group| {
///         radio_button(selected.get() == 0)
///             .group(group)
///             .on_select(move |_| selected.set(0));
///         radio_button(selected.get() == 1)
///             .group(group)
///             .on_select(move |_| selected.set(1));
///     });
/// }
/// ```
#[tessera]
pub fn radio_group(
    modifier: Option<Modifier>,
    orientation: Option<RadioGroupOrientation>,
    wrap: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    content: Option<RenderSlotWith<RadioGroupScope>>,
) {
    let modifier = modifier.unwrap_or_default();
    let wrap = wrap.unwrap_or(true);
    let orientation = match orientation.unwrap_or_default() {
        RadioGroupOrientation::Horizontal => Orientation::Horizontal,
        RadioGroupOrientation::Vertical => Orientation::Vertical,
    };
    let modifier = modifier.focus_group().focus_traversal_policy(
        FocusTraversalPolicy::linear()
            .wrap(wrap)
            .single_tab_stop(true),
    );
    let modifier = modifier.push_semantics(RadioSemantics {
        group: true,
        selected: false,
        label: accessibility_label,
        description: accessibility_description,
        orientation: Some(orientation),
    });
    layout(Stack).modifier(modifier).child(move || {
        if let Some(content) = content {
            content.render(RadioGroupScope(()));
        }
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
/// - `focus_requester` — optional handle targeting this option for focus
///   requests.
/// - `on_select` — optional callback invoked when the radio transitions to the
///   selected state.
/// - `selected` — whether this choice is selected.
/// - `group` — scope received from the enclosing radio group.
/// - `size` — optional visual diameter of the radio glyph.
/// - `touch_target_size` — optional minimum interactive touch target.
/// - `stroke_width` — optional stroke width applied to the outer ring.
/// - `dot_size` — optional diameter of the inner dot when fully selected.
/// - `selected_color` — optional ring and dot color when selected.
/// - `unselected_color` — optional ring color override for all unselected
///   states.
/// - `disabled_selected_color` — optional ring and dot color when disabled but
///   selected.
/// - `disabled_unselected_color` — optional ring color when disabled and not
///   selected.
/// - `enabled` — whether the control is interactive.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
///
/// ## Examples
///
/// ```rust
/// use tessera_components::radio_button::radio_button;
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn radio_demo() {
///     radio_button(true);
/// }
/// ```
#[tessera]
pub fn radio_button(
    selected: bool,
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    on_select: Option<CallbackWith<bool, ()>>,
    group: Option<RadioGroupScope>,
    size: Option<Dp>,
    touch_target_size: Option<Dp>,
    stroke_width: Option<Dp>,
    dot_size: Option<Dp>,
    selected_color: Option<Color>,
    unselected_color: Option<Color>,
    disabled_selected_color: Option<Color>,
    disabled_unselected_color: Option<Color>,
    enabled: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let modifier = modifier.unwrap_or_default();
    let enabled = enabled.unwrap_or(true);
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let size = size.unwrap_or(Dp(20.0));
    let touch_target_size = touch_target_size.unwrap_or(Dp(48.0));
    let stroke_width = stroke_width.unwrap_or(Dp(2.0));
    let dot_size = dot_size.unwrap_or(Dp(10.0));
    let selected_color = selected_color.unwrap_or(scheme.primary);
    let unselected_interaction_color = unselected_color.unwrap_or(scheme.on_surface);
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

    let controller = remember(|| RadioButtonMotion::new(selected));
    if controller.with(|c| c.is_selected()) != selected {
        controller.with_mut(|c| c.set_selected(selected));
    }

    radio_button_inner()
        .focus_requester_optional(focus_requester)
        .modifier(modifier)
        .on_select_shared(on_select)
        .group_optional(group)
        .size(size)
        .touch_target_size(touch_target_size)
        .stroke_width(stroke_width)
        .dot_size(dot_size)
        .selected_color(selected_color)
        .unselected_color(unselected_color)
        .unselected_interaction_color(unselected_interaction_color)
        .disabled_selected_color(disabled_selected_color)
        .disabled_unselected_color(disabled_unselected_color)
        .enabled(enabled)
        .controller(controller)
        .accessibility_label_optional(accessibility_label)
        .accessibility_description_optional(accessibility_description);
}

#[tessera]
fn radio_button_inner(
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    on_select: Option<CallbackWith<bool, ()>>,
    group: Option<RadioGroupScope>,
    size: Option<Dp>,
    touch_target_size: Option<Dp>,
    stroke_width: Option<Dp>,
    dot_size: Option<Dp>,
    selected_color: Option<Color>,
    unselected_color: Option<Color>,
    unselected_interaction_color: Option<Color>,
    disabled_selected_color: Option<Color>,
    disabled_unselected_color: Option<Color>,
    enabled: Option<bool>,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    controller: Option<State<RadioButtonMotion>>,
) {
    let modifier = modifier.unwrap_or_default();
    let size = size.unwrap_or(Dp(20.0));
    let touch_target_size = touch_target_size.unwrap_or(Dp(48.0));
    let stroke_width = stroke_width.unwrap_or(Dp(2.0));
    let dot_size = dot_size.unwrap_or(Dp(10.0));
    let selected_color = selected_color.unwrap_or(Color::TRANSPARENT);
    let unselected_color = unselected_color.unwrap_or(Color::TRANSPARENT);
    let unselected_interaction_color = unselected_interaction_color.unwrap_or(unselected_color);
    let disabled_selected_color = disabled_selected_color.unwrap_or(Color::TRANSPARENT);
    let disabled_unselected_color = disabled_unselected_color.unwrap_or(Color::TRANSPARENT);
    let enabled = enabled.unwrap_or(true);
    let radio_group = group;
    let controller = controller.expect("radio_button_inner requires controller to be set");
    let interaction_state = if enabled {
        Some(remember(InteractionState::new))
    } else {
        None
    };
    let ripple_state = if enabled {
        Some(remember(RippleState::new))
    } else {
        None
    };
    let tap_recognizer = if enabled {
        Some(remember(TapRecognizer::default))
    } else {
        None
    };
    if controller.with(|c| c.is_animating()) {
        // Rebuilds this component on every animated frame.
        let frame_clock = tessera_ui::remember(tessera_ui::current_frame_nanos);
        let _ = frame_clock.get();
        tessera_ui::launch_once(async move {
            let step = move |frame_nanos: u64| {
                let is_animating = controller.with_mut(|controller| {
                    controller.update_animation(frame_nanos);
                    controller.is_animating()
                });
                if is_animating { true } else { false }
            };
            loop {
                let now = tessera_ui::next_frame().await;
                frame_clock.set(now);
                if !step(now) {
                    break;
                }
            }
        });
    }
    let progress = controller.with(|c| c.animation_progress());
    let eased_progress = animation::easing(progress);
    let is_selected = controller.with(|c| c.is_selected());
    let on_select = on_select.unwrap_or_else(CallbackWith::default_value);

    let target_size = Dp(touch_target_size.0.max(size.0));

    let unselected_color = if enabled
        && interaction_state.is_some_and(|state| {
            state.with(|state| state.is_hovered() || state.is_focused() || state.is_pressed())
        }) {
        unselected_interaction_color
    } else {
        unselected_color
    };
    let ring_color = if enabled {
        interpolate_color(unselected_color, selected_color, progress)
    } else if is_selected {
        disabled_selected_color
    } else {
        disabled_unselected_color
    };

    let ripple_color = ring_color;

    let target_dot_color = if enabled {
        selected_color
    } else {
        disabled_selected_color
    };
    let active_dot_color = interpolate_color(Color::TRANSPARENT, target_dot_color, eased_progress);

    let on_click = Callback::new(move || {
        if !is_selected {
            on_select.call(true);
        }
    });

    let state_layer_size = RadioButtonDefaults::STATE_LAYER_SIZE;
    let state_layer_radius = Dp(state_layer_size.0 / 2.0);

    let mut modifier = modifier.size(target_size, target_size);
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
            focus_requester,
            selected: is_selected,
            on_click,
            enabled: true,
            role: Some(Role::RadioButton),
            label: accessibility_label.clone(),
            description: accessibility_description.clone(),
            interaction_state,
            on_press: press_handler.map(Into::into),
            on_release: release_handler.map(Into::into),
            tap_recognizer,
            ..Default::default()
        };
        modifier = modifier.selectable_with(selectable_args);
        if radio_group.is_some() {
            modifier = modifier.preferred_tab_stop(is_selected).on_focus_changed(
                move |focus_state: FocusState| {
                    let pointer_pressed = interaction_state
                        .is_some_and(|state| state.with(|state| state.is_pressed()));
                    if let Some(interaction_state) = interaction_state {
                        interaction_state
                            .with_mut(|state| state.set_focused(focus_state.has_focus()));
                    }
                    let directional = matches!(
                        focus::focus_change_direction(),
                        Some(
                            FocusDirection::Left
                                | FocusDirection::Right
                                | FocusDirection::Up
                                | FocusDirection::Down
                        )
                    );
                    if focus_state.is_focused() && directional && !pointer_pressed {
                        on_click.call();
                    }
                },
            );
        }
    } else {
        modifier = modifier.push_semantics(RadioSemantics {
            group: false,
            selected: is_selected,
            label: accessibility_label,
            description: accessibility_description,
            orientation: None,
        });
    }

    boxed()
        .modifier(modifier)
        .alignment(Alignment::Center)
        .children(move || {
            surface()
                .modifier(Modifier::new().size(state_layer_size, state_layer_size))
                .shape(Shape::Ellipse)
                .enabled(enabled)
                .style(SurfaceStyle::Filled {
                    color: Color::TRANSPARENT,
                })
                .ripple_bounded(false)
                .ripple_radius(state_layer_radius)
                .ripple_color(ripple_color)
                .interaction_state_optional(interaction_state)
                .ripple_state_optional(ripple_state)
                .child({
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
                                    .child({
                                        move || {
                                            let animated_size = (dot_size.to_px().0 as f32
                                                * eased_progress)
                                                .round()
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
                                                            .child(|| {});
                                                    });
                                            }
                                        }
                                    });
                            });
                    }
                });
        });
}
