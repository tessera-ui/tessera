//! Fluid glass switch - toggle state with a surface track and glass lens.
//!
//! ## Usage
//!
//! Toggle boolean state over visible surface-backed content.
use std::time::Duration;

use tessera_foundation::gesture::TapRecognizer;
use tessera_ui::{
    CallbackWith, Color, ComputedData, Constraint, Dp, LayoutResult, MeasurementError, Modifier,
    Px, PxPosition, State,
    accesskit::{Role, Toggled},
    current_frame_nanos,
    layout::{LayoutPolicy, MeasureScope, PlacementScope, layout},
    receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    animation,
    fluid_glass::fluid_glass,
    modifier::{
        InteractionState, ModifierExt as _, PointerEventContext, SemanticsArgs, ToggleableArgs,
    },
    shape_def::Shape,
    surface::{SurfaceStyle, surface},
    theme::{MaterialAlpha, MaterialColorScheme, MaterialTheme},
};

const TRAVEL_DURATION: Duration = Duration::from_millis(200);
const EXPAND_BEFORE_TRAVEL_DURATION: Duration = Duration::from_millis(90);
const SPRING_DURATION: Duration = Duration::from_millis(260);
const TRAVEL_WITHOUT_DELAY_EXPANSION: f32 = 0.75;

#[cfg(test)]
const LENS_TEST_TAG: &str = "__fluid_switch_lens";

/// Defaults for [`fluid_switch`].
pub struct FluidSwitchDefaults;

impl FluidSwitchDefaults {
    /// Default track width.
    pub const TRACK_WIDTH: Dp = Dp(91.0);
    /// Default track height.
    pub const TRACK_HEIGHT: Dp = Dp(32.0);
    /// Resting lens width.
    pub const LENS_WIDTH: Dp = Dp(54.0);
    /// Resting lens height.
    pub const LENS_HEIGHT: Dp = Dp(32.0);
    /// Pressed or traveling lens width.
    pub const EXPANDED_LENS_WIDTH: Dp = Dp(72.9);
    /// Pressed or traveling lens height.
    pub const EXPANDED_LENS_HEIGHT: Dp = Dp(43.2);
    /// Track outline width.
    pub const TRACK_OUTLINE_WIDTH: Dp = Dp(1.0);
    /// Default checked substrate tint.
    pub const CHECKED_TINT_COLOR: Color = Color::new(0.34, 0.82, 0.42, 1.0);
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SpringTransition {
    start_frame_nanos: u64,
    from: f32,
    to: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TravelTransition {
    start_frame_nanos: u64,
    from: f32,
    to: f32,
}

struct FluidSwitchState {
    checked: bool,
    travel_progress: f32,
    expansion_progress: f32,
    travel_transition: Option<TravelTransition>,
    expansion_transition: Option<SpringTransition>,
}

impl FluidSwitchState {
    fn new(initial_state: bool) -> Self {
        Self {
            checked: initial_state,
            travel_progress: if initial_state { 1.0 } else { 0.0 },
            expansion_progress: 0.0,
            travel_transition: None,
            expansion_transition: None,
        }
    }

    fn is_checked(&self) -> bool {
        self.checked
    }

    fn snap_to_checked(&mut self, checked: bool) {
        self.checked = checked;
        self.travel_progress = if checked { 1.0 } else { 0.0 };
        self.expansion_progress = 0.0;
        self.travel_transition = None;
        self.expansion_transition = None;
    }

    fn travel_progress(&self) -> f32 {
        self.travel_progress
    }

    fn expansion_progress(&self) -> f32 {
        self.expansion_progress
    }

    fn is_animating(&self) -> bool {
        self.travel_transition.is_some() || self.expansion_transition.is_some()
    }

    fn press(&mut self) {
        self.retarget_expansion(1.0, current_frame_nanos());
    }

    fn release(&mut self) {
        self.retarget_expansion(0.0, current_frame_nanos());
    }

    fn toggle_at(&mut self, frame_nanos: u64) {
        self.update(frame_nanos);
        self.checked = !self.checked;
        self.retarget_expansion(1.0, frame_nanos);

        let travel_delay = if self.expansion_progress >= TRAVEL_WITHOUT_DELAY_EXPANSION {
            0
        } else {
            duration_nanos(EXPAND_BEFORE_TRAVEL_DURATION)
        };

        self.travel_transition = Some(TravelTransition {
            start_frame_nanos: frame_nanos.saturating_add(travel_delay),
            from: self.travel_progress,
            to: if self.checked { 1.0 } else { 0.0 },
        });
    }

    fn animate_to_checked(&mut self, checked: bool) {
        if self.checked != checked {
            self.toggle_at(current_frame_nanos());
        }
    }

    fn update(&mut self, frame_nanos: u64) {
        self.update_expansion(frame_nanos);
        self.update_travel(frame_nanos);
    }

    fn update_expansion(&mut self, frame_nanos: u64) {
        let Some(transition) = self.expansion_transition else {
            return;
        };
        let elapsed = frame_nanos.saturating_sub(transition.start_frame_nanos);
        let duration = duration_nanos(SPRING_DURATION);
        if elapsed >= duration {
            self.expansion_progress = transition.to;
            self.expansion_transition = None;
            return;
        }

        let fraction = elapsed as f32 / duration as f32;
        let spring = spring_progress(fraction);
        self.expansion_progress = transition.from + (transition.to - transition.from) * spring;
    }

    fn update_travel(&mut self, frame_nanos: u64) {
        let Some(transition) = self.travel_transition else {
            return;
        };

        if frame_nanos < transition.start_frame_nanos {
            self.travel_progress = transition.from;
            return;
        }

        let elapsed = frame_nanos.saturating_sub(transition.start_frame_nanos);
        let duration = duration_nanos(TRAVEL_DURATION);
        if elapsed >= duration {
            self.travel_progress = transition.to;
            self.travel_transition = None;
            self.retarget_expansion(0.0, frame_nanos);
            return;
        }

        let fraction = elapsed as f32 / duration as f32;
        self.travel_progress = transition.from + (transition.to - transition.from) * fraction;
    }

    fn retarget_expansion(&mut self, target: f32, frame_nanos: u64) {
        self.update_expansion(frame_nanos);
        let current = self.expansion_progress;
        if (current - target).abs() <= f32::EPSILON {
            self.expansion_progress = target;
            self.expansion_transition = None;
            return;
        }

        self.expansion_transition = Some(SpringTransition {
            start_frame_nanos: frame_nanos,
            from: current,
            to: target,
        });
    }
}

impl Default for FluidSwitchState {
    fn default() -> Self {
        Self::new(false)
    }
}

fn duration_nanos(duration: Duration) -> u64 {
    duration.as_nanos().min(u64::MAX as u128) as u64
}

fn spring_progress(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    if progress >= 1.0 {
        return 1.0;
    }

    1.0 - (-6.0 * progress).exp() * (10.0 * progress).cos()
}

fn lerp_dp(from: Dp, to: Dp, progress: f32) -> Dp {
    Dp(from.0 + (to.0 - from.0) * progress as f64)
}

fn resolve_track_color(
    scheme: &MaterialColorScheme,
    accent_color: Color,
    enabled: bool,
    progress: f32,
) -> Color {
    if !enabled {
        return scheme
            .surface
            .blend_over(scheme.on_surface, MaterialAlpha::DISABLED_CONTAINER * 0.5);
    }

    let off = if scheme.is_dark {
        Color::new(0.18, 0.19, 0.18, 1.0)
    } else {
        Color::new(0.92, 0.92, 0.92, 1.0)
    };
    let on = if scheme.is_dark {
        accent_color.lerp(&Color::new(0.0, 0.0, 0.0, 1.0), 0.18)
    } else {
        accent_color.lerp(&Color::new(1.0, 1.0, 1.0, 1.0), 0.32)
    };
    off.lerp(&on, progress)
}

fn resolve_track_outline_color(
    scheme: &MaterialColorScheme,
    accent_color: Color,
    enabled: bool,
    progress: f32,
) -> Color {
    if !enabled {
        return scheme.outline.with_alpha(MaterialAlpha::DISABLED_CONTAINER);
    }

    let off = scheme.outline_variant.with_alpha(0.22);
    let on = accent_color.with_alpha(0.7);
    off.lerp(&on, progress)
}

fn resolve_source_checked(checked_prop: Option<bool>, visual_checked: bool) -> bool {
    checked_prop.unwrap_or(visual_checked)
}

fn update_state_for_frame(state: State<FluidSwitchState>, frame_nanos: u64) -> bool {
    state.with_mut(|state| {
        state.update(frame_nanos);
        state.is_animating()
    })
}

/// # fluid_switch
///
/// Render a fluid glass switch for prominent boolean toggles over visible
/// surface-backed content.
///
/// ## Usage
///
/// Use for settings or mode toggles where the moving glass lens is the primary
/// interaction cue.
///
/// ## Parameters
///
/// - `modifier` - optional modifier chain applied to the switch subtree.
/// - `on_toggle` - optional callback invoked when the switch toggles.
/// - `enabled` - optional enabled state; defaults to `true`.
/// - `checked` - optional external checked state. When omitted, the switch
///   manages its own state internally.
/// - `accent_color` - optional checked-state tint blended into the surface
///   substrate; defaults to a pale green.
/// - `accessibility_label` - optional accessibility label.
/// - `accessibility_description` - optional accessibility description.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     fluid_switch::fluid_switch,
///     surface::surface,
///     theme::{MaterialTheme, material_theme},
/// };
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// let checked = tessera_ui::remember(|| false);
/// assert!(!checked.get());
///
/// material_theme()
///     .theme(|| MaterialTheme::default())
///     .child(move || {
///         surface().child(move || {
///             fluid_switch()
///                 .checked(checked.get())
///                 .on_toggle(move |next_checked| checked.set(next_checked));
///         });
///     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn fluid_switch(
    modifier: Option<Modifier>,
    on_toggle: Option<CallbackWith<bool, ()>>,
    enabled: Option<bool>,
    checked: Option<bool>,
    accent_color: Option<Color>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let checked_prop = checked;
    let initial_checked = checked_prop.unwrap_or(false);
    let accent_color = accent_color.unwrap_or(FluidSwitchDefaults::CHECKED_TINT_COLOR);
    let state = remember(|| FluidSwitchState::new(initial_checked));

    fluid_switch_content()
        .modifier_optional(modifier)
        .on_toggle_optional(on_toggle)
        .enabled(enabled)
        .checked_optional(checked_prop)
        .accent_color(accent_color)
        .accessibility_label_optional(accessibility_label)
        .accessibility_description_optional(accessibility_description)
        .state(state);
}

#[tessera]
#[allow(clippy::too_many_arguments)]
fn fluid_switch_content(
    modifier: Option<Modifier>,
    on_toggle: Option<CallbackWith<bool, ()>>,
    enabled: Option<bool>,
    checked: Option<bool>,
    accent_color: Option<Color>,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    state: Option<State<FluidSwitchState>>,
) {
    let enabled = enabled.unwrap_or(true);
    let checked_prop = checked;
    let accent_color = accent_color.unwrap_or(FluidSwitchDefaults::CHECKED_TINT_COLOR);
    let state = state.expect("fluid_switch_content requires state to be set");
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let last_checked_prop = remember(|| Option::<bool>::None);

    let previous_checked_prop = last_checked_prop.get();
    if let Some(checked) = checked_prop {
        if previous_checked_prop.is_none() {
            state.with_mut(|state| state.snap_to_checked(checked));
        } else if previous_checked_prop != checked_prop {
            state.with_mut(|state| state.animate_to_checked(checked));
        } else if state.with(|state| !state.is_animating() && state.is_checked() != checked) {
            state.with_mut(|state| state.animate_to_checked(checked));
        }
    }
    if previous_checked_prop != checked_prop {
        last_checked_prop.set(checked_prop);
    }

    let mut modifier = modifier.unwrap_or_default();
    let visual_checked = state.with(|state| state.is_checked());
    let source_checked = resolve_source_checked(checked_prop, visual_checked);
    let travel_progress = state.with(|state| state.travel_progress());
    let eased_progress = animation::easing(travel_progress);
    let expansion_progress = state.with(|state| state.expansion_progress());
    let track_color = resolve_track_color(&scheme, accent_color, enabled, eased_progress);
    let track_outline_color =
        resolve_track_outline_color(&scheme, accent_color, enabled, eased_progress);
    let has_toggle_handler = on_toggle.is_some();
    let interaction_state = remember(InteractionState::new);
    let tap_recognizer = remember(TapRecognizer::default);

    if has_toggle_handler {
        modifier = modifier.minimum_interactive_component_size();
        let press_handler = move |_ctx: PointerEventContext| {
            state.with_mut(|state| state.press());
        };
        let release_handler = move |_ctx: PointerEventContext| {
            state.with_mut(|state| state.release());
        };
        let toggle_args = ToggleableArgs {
            value: source_checked,
            on_value_change: CallbackWith::new(move |next_checked| {
                state.with_mut(|state| state.animate_to_checked(next_checked));
                if let Some(on_toggle) = on_toggle.as_ref() {
                    on_toggle.call(next_checked);
                }
            }),
            enabled,
            role: Some(Role::Switch),
            label: accessibility_label.clone(),
            description: accessibility_description.clone(),
            interaction_state: Some(interaction_state),
            on_press: Some(press_handler.into()),
            on_release: Some(release_handler.into()),
            tap_recognizer: Some(tap_recognizer),
            ..Default::default()
        };
        modifier = modifier.toggleable_with(toggle_args);
    } else {
        modifier = modifier.semantics(SemanticsArgs {
            role: Some(Role::Switch),
            label: accessibility_label.clone(),
            description: accessibility_description.clone(),
            toggled: Some(if source_checked {
                Toggled::True
            } else {
                Toggled::False
            }),
            disabled: !enabled,
            ..Default::default()
        });
    }

    let should_continue = update_state_for_frame(state, current_frame_nanos());
    if should_continue {
        receive_frame_nanos(move |frame_nanos| {
            if update_state_for_frame(state, frame_nanos) {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    let lens_width = lerp_dp(
        FluidSwitchDefaults::LENS_WIDTH,
        FluidSwitchDefaults::EXPANDED_LENS_WIDTH,
        expansion_progress,
    );
    let lens_height = lerp_dp(
        FluidSwitchDefaults::LENS_HEIGHT,
        FluidSwitchDefaults::EXPANDED_LENS_HEIGHT,
        expansion_progress,
    );

    layout()
        .modifier(modifier)
        .layout_policy(FluidSwitchLayout {
            track_width: FluidSwitchDefaults::TRACK_WIDTH.to_px(),
            track_height: FluidSwitchDefaults::TRACK_HEIGHT.to_px(),
            lens_width: lens_width.to_px(),
            lens_height: lens_height.to_px(),
            progress: travel_progress,
        })
        .child(move || {
            surface()
                .modifier(Modifier::new().size(
                    FluidSwitchDefaults::TRACK_WIDTH,
                    FluidSwitchDefaults::TRACK_HEIGHT,
                ))
                .style(SurfaceStyle::FilledOutlined {
                    fill_color: track_color,
                    border_color: track_outline_color,
                    border_width: FluidSwitchDefaults::TRACK_OUTLINE_WIDTH,
                })
                .shape(Shape::CAPSULE)
                .show_state_layer(false)
                .show_ripple(false)
                .child(|| {});

            let lens_modifier = Modifier::new().size(lens_width, lens_height);
            #[cfg(test)]
            let lens_modifier = lens_modifier.semantics(SemanticsArgs {
                test_tag: Some(LENS_TEST_TAG.to_string()),
                ..Default::default()
            });

            fluid_glass()
                .modifier(lens_modifier)
                .shape(Shape::CAPSULE)
                .block_input(false);
        });
}

#[derive(Clone, PartialEq)]
struct FluidSwitchLayout {
    track_width: Px,
    track_height: Px,
    lens_width: Px,
    lens_height: Px,
    progress: f32,
}

impl LayoutPolicy for FluidSwitchLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        if children.len() < 2 {
            return Ok(result.with_size(ComputedData {
                width: self.track_width,
                height: self.track_height.max(self.lens_height),
            }));
        }

        let track = children[0];
        let lens = children[1];
        let track_constraint = Constraint::exact(self.track_width, self.track_height);
        let lens_constraint = Constraint::exact(self.lens_width, self.lens_height);

        let track_size = track.measure(&track_constraint)?;
        let lens_size = lens.measure(&lens_constraint)?;
        let width = track_size.width.max(lens_size.width);
        let height = track_size.height.max(lens_size.height);
        let track_x = (width - track_size.width) / 2;
        let track_y = (height - track_size.height) / 2;
        let lens_y = (height - lens_size.height) / 2;
        let lens_x = compute_lens_x(track_x, track_size.width, lens_size.width, self.progress);

        result.place_child(track, PxPosition::new(track_x, track_y));
        result.place_child(lens, PxPosition::new(lens_x, lens_y));

        Ok(result.with_size(ComputedData { width, height }))
    }

    fn measure_eq(&self, other: &Self) -> bool {
        self.track_width == other.track_width
            && self.track_height == other.track_height
            && self.lens_width == other.lens_width
            && self.lens_height == other.lens_height
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.measure_eq(other) && self.progress == other.progress
    }

    fn place_children(&self, input: &PlacementScope<'_>) -> Option<Vec<(u64, PxPosition)>> {
        let mut result = LayoutResult::default();
        let children = input.children();
        if children.len() < 2 {
            return Some(result.into_placements());
        }

        let track = children[0];
        let lens = children[1];
        let track_size = track.size();
        let lens_size = lens.size();
        let width = input.size().width;
        let height = input.size().height;
        let track_x = (width - track_size.width) / 2;
        let track_y = (height - track_size.height) / 2;
        let lens_y = (height - lens_size.height) / 2;
        let lens_x = compute_lens_x(track_x, track_size.width, lens_size.width, self.progress);

        result.place_child(track, PxPosition::new(track_x, track_y));
        result.place_child(lens, PxPosition::new(lens_x, lens_y));
        Some(result.into_placements())
    }
}

fn compute_lens_x(track_x: Px, track_width: Px, lens_width: Px, progress: f32) -> Px {
    let eased_progress = animation::easing(progress);
    let start = track_x.0;
    let end = track_x.0 + track_width.0 - lens_width.0;
    Px(start + ((end - start) as f32 * eased_progress).round() as i32)
}

#[cfg(test)]
mod tests {
    use tessera_ui::tessera;

    use crate::{
        fluid_switch::{
            EXPAND_BEFORE_TRAVEL_DURATION, FluidSwitchDefaults, FluidSwitchState, LENS_TEST_TAG,
            SPRING_DURATION, TRAVEL_DURATION, fluid_switch_content, resolve_source_checked,
        },
        theme::{MaterialTheme, material_theme},
    };

    #[tessera]
    fn animated_lens_probe() {
        let state = tessera_ui::remember(|| FluidSwitchState::new(false));
        let started = tessera_ui::remember(|| false);

        if !started.get() {
            state.with_mut(|state| state.toggle_at(0));
            started.set(true);
        }

        state.with_mut(|state| {
            state.update(tessera_ui::current_frame_nanos());
        });

        material_theme()
            .theme(MaterialTheme::default)
            .child(move || {
                fluid_switch_content()
                    .on_toggle(|_| {})
                    .enabled(true)
                    .accent_color(FluidSwitchDefaults::CHECKED_TINT_COLOR)
                    .state(state);
            });
    }

    #[test]
    fn fluid_switch_state_delays_travel_until_lens_expands() {
        let mut state = FluidSwitchState::new(false);

        state.toggle_at(0);

        assert!(state.is_checked());
        assert!(state.is_animating());
        assert_eq!(state.travel_progress(), 0.0);

        let delay_nanos = EXPAND_BEFORE_TRAVEL_DURATION.as_nanos() as u64;
        state.update(delay_nanos - 1);
        assert_eq!(state.travel_progress(), 0.0);
        assert!(state.expansion_progress() > 0.0);

        state.update(delay_nanos + (TRAVEL_DURATION.as_nanos() / 2) as u64);
        let mid = state.travel_progress();
        assert!(mid > 0.0 && mid < 1.0, "mid travel progress was {mid}");

        state.update(delay_nanos + TRAVEL_DURATION.as_nanos() as u64);
        assert_eq!(state.travel_progress(), 1.0);
        assert!(state.expansion_progress() > 0.0);

        state.update(
            delay_nanos + TRAVEL_DURATION.as_nanos() as u64 + SPRING_DURATION.as_nanos() as u64,
        );
        assert_eq!(state.expansion_progress(), 0.0);
        assert!(!state.is_animating());
    }

    #[test]
    fn fluid_switch_state_ignores_redundant_animate_target() {
        let mut state = FluidSwitchState::new(false);

        state.animate_to_checked(false);

        assert!(!state.is_checked());
        assert!(!state.is_animating());

        state.animate_to_checked(true);

        assert!(state.is_checked());
        assert!(state.is_animating());
    }

    #[test]
    fn fluid_switch_controlled_interaction_uses_checked_prop_as_source() {
        assert!(!resolve_source_checked(Some(false), true));
        assert!(resolve_source_checked(Some(true), false));
        assert!(resolve_source_checked(None, true));
        assert!(!resolve_source_checked(None, false));
    }

    #[test]
    fn fluid_switch_lens_expands_before_travel() {
        let delay_nanos = EXPAND_BEFORE_TRAVEL_DURATION.as_nanos() as u64;
        tessera_ui::assert_layout! {
            viewport: (200, 100),
            content: {
                animated_lens_probe();
            },
            expect: {
                0 => {
                    node(LENS_TEST_TAG).size(54, 32);
                },
                delay_nanos => {
                    node(LENS_TEST_TAG).size(75, 44);
                }
            }
        }
    }
}
