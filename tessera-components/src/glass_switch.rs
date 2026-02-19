//! A switch (toggle) component with a glassmorphic visual style.
//!
//! ## Usage
//!
//! Use in settings, forms, or toolbars to control a boolean state.
use std::time::{Duration, Instant};

use derive_setters::Setters;
use tessera_ui::{
    CallbackWith, Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Modifier,
    Px, PxPosition, State,
    accesskit::Role,
    layout::{LayoutInput, LayoutOutput, LayoutSpec},
    receive_frame_nanos, remember, tessera,
};

use crate::{
    animation,
    fluid_glass::{FluidGlassArgs, GlassBorder, fluid_glass},
    modifier::{InteractionState, ModifierExt as _, ToggleableArgs},
    shape_def::Shape,
};

const ANIMATION_DURATION: Duration = Duration::from_millis(150);

/// Controller for the `glass_switch` component.
#[derive(Clone, PartialEq)]
pub struct GlassSwitchController {
    checked: bool,
    progress: f32,
    last_toggle_time: Option<Instant>,
}

impl GlassSwitchController {
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

    /// Toggles the switch and starts the animation timeline.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
        self.last_toggle_time = Some(Instant::now());
    }

    /// Returns the current animation progress (0.0..1.0).
    pub fn animation_progress(&mut self) -> f32 {
        if let Some(start) = self.last_toggle_time {
            let elapsed = start.elapsed();
            let fraction = (elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32()).min(1.0);
            let target = if self.checked { 1.0 } else { 0.0 };
            self.progress = target * fraction + (1.0 - fraction) * (1.0 - target);
            if fraction >= 1.0 {
                self.last_toggle_time = None;
                self.progress = target;
            }
        }
        self.progress
    }

    /// Returns whether the toggle animation is currently running.
    pub fn is_animating(&self) -> bool {
        self.last_toggle_time.is_some()
    }
}

impl Default for GlassSwitchController {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Arguments for the `glass_switch` component.
#[derive(PartialEq, Clone, Setters)]
pub struct GlassSwitchArgs {
    /// Optional modifier chain applied to the switch subtree.
    pub modifier: Modifier,
    /// Optional callback invoked when the switch toggles.
    #[setters(skip)]
    pub on_toggle: Option<CallbackWith<bool, ()>>,
    /// Initial checked state.
    pub checked: bool,

    /// Total width of the switch track.
    pub width: Dp,

    /// Total height of the switch track (including padding).
    pub height: Dp,

    /// Track color when switch is ON
    pub track_on_color: Color,
    /// Track color when switch is OFF
    pub track_off_color: Color,

    /// Thumb alpha when switch is ON (opacity when ON)
    pub thumb_on_alpha: f32,
    /// Thumb alpha when switch is OFF (opacity when OFF)
    pub thumb_off_alpha: f32,

    /// Border for the thumb
    #[setters(strip_option)]
    pub thumb_border: Option<GlassBorder>,

    /// Border for the track
    #[setters(strip_option)]
    pub track_border: Option<GlassBorder>,

    /// Padding around the thumb
    pub thumb_padding: Dp,
    /// Optional accessibility label read by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
    /// Optional external controller for checked state and animation.
    ///
    /// When this is `None`, `glass_switch` creates and owns an internal
    /// controller.
    #[setters(skip)]
    pub controller: Option<State<GlassSwitchController>>,
}

impl GlassSwitchArgs {
    /// Sets the on_toggle handler.
    pub fn on_toggle<F>(mut self, on_toggle: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_toggle = Some(CallbackWith::new(on_toggle));
        self
    }

    /// Sets the on_toggle handler using a shared callback.
    pub fn on_toggle_shared(mut self, on_toggle: impl Into<CallbackWith<bool, ()>>) -> Self {
        self.on_toggle = Some(on_toggle.into());
        self
    }

    /// Sets an external glass switch controller.
    pub fn controller(mut self, controller: State<GlassSwitchController>) -> Self {
        self.controller = Some(controller);
        self
    }
}

impl Default for GlassSwitchArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new(),
            on_toggle: None,
            checked: false,
            width: Dp(52.0),
            height: Dp(32.0),
            track_on_color: Color::new(0.2, 0.7, 1.0, 0.5),
            track_off_color: Color::new(0.8, 0.8, 0.8, 0.5),
            thumb_on_alpha: 0.5,
            thumb_off_alpha: 1.0,
            thumb_border: None,
            track_border: None,
            thumb_padding: Dp(3.0),
            accessibility_label: None,
            accessibility_description: None,
            controller: None,
        }
    }
}

fn interpolate_color(off: Color, on: Color, progress: f32) -> Color {
    Color {
        r: off.r + (on.r - off.r) * progress,
        g: off.g + (on.g - off.g) * progress,
        b: off.b + (on.b - off.b) * progress,
        a: off.a + (on.a - off.a) * progress,
    }
}

/// # glass_switch
///
/// An interactive switch with glass effect.
///
/// ## Usage
///
/// Use when you need a toggle switch with a glassmorphic style.
///
/// ## Parameters
///
/// - `args` — configures the switch's appearance and `on_toggle` callback; see
///   [`GlassSwitchArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_components::glass_switch::{GlassSwitchArgs, GlassSwitchController, glass_switch};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let controller = remember(|| GlassSwitchController::new(false));
///     assert!(!controller.with(|c| c.is_checked()));
///
///     let args = GlassSwitchArgs::default().controller(controller);
///     glass_switch(&args);
///
///     controller.with_mut(|c| c.toggle());
///     assert!(controller.with(|c| c.is_checked()));
/// }
///
/// demo();
/// ```
#[tessera]
pub fn glass_switch(args: &GlassSwitchArgs) {
    let mut switch_args = args.clone();
    let controller = switch_args
        .controller
        .unwrap_or_else(|| remember(|| GlassSwitchController::new(switch_args.checked)));
    if controller.with(|c| c.is_checked()) != switch_args.checked {
        controller.with_mut(|c| c.set_checked(switch_args.checked));
    }
    switch_args.controller = Some(controller);
    glass_switch_node(&switch_args);
}

#[tessera]
fn glass_switch_node(args: &GlassSwitchArgs) {
    let args = args.clone();
    let controller = args
        .controller
        .expect("glass_switch_node requires controller to be set");
    let mut modifier = args.modifier;

    let on_toggle = args.on_toggle.clone();
    let enabled = on_toggle.is_some();
    let interaction_state = enabled.then(|| remember(InteractionState::new));
    let checked = controller.with(|c| c.is_checked());
    if enabled {
        modifier = modifier.minimum_interactive_component_size();
        let on_toggle = on_toggle.clone();
        let mut toggle_args = ToggleableArgs::new(checked, move |_| {
            controller.with_mut(|c| c.toggle());
            let checked = controller.with(|c| c.is_checked());
            if let Some(on_toggle) = on_toggle.as_ref() {
                on_toggle.call(checked);
            }
        })
        .enabled(true)
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
        modifier = modifier.toggleable(toggle_args);
    }

    // Precompute pixel sizes to avoid repeated conversions
    let width_px = args.width.to_px();
    let height_px = args.height.to_px();
    let thumb_dp = Dp(args.height.0 - (args.thumb_padding.0 * 2.0));
    let thumb_px = thumb_dp.to_px();

    // Track tint color interpolation based on progress
    let progress = controller.with_mut(|c| c.animation_progress());
    if controller.with(|c| c.is_animating()) {
        let controller_for_frame = controller;
        receive_frame_nanos(move |_| {
            let is_animating = controller_for_frame.with_mut(|controller| {
                let _ = controller.animation_progress();
                controller.is_animating()
            });
            if is_animating {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let track_color = interpolate_color(args.track_off_color, args.track_on_color, progress);

    modifier.run(move || {
        // Build and render track
        let mut track_args = FluidGlassArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::Fixed(width_px)),
                Some(DimensionValue::Fixed(height_px)),
            ))
            .tint_color(track_color)
            .shape(Shape::capsule())
            .blur_radius(8.0);
        if let Some(border) = args.track_border {
            track_args = track_args.border(border);
        }
        fluid_glass(&crate::fluid_glass::FluidGlassArgs::with_child(
            &track_args,
            || {},
        ));

        // Build and render thumb
        let thumb_alpha =
            args.thumb_off_alpha + (args.thumb_on_alpha - args.thumb_off_alpha) * progress;
        let thumb_color = Color::new(1.0, 1.0, 1.0, thumb_alpha);
        let mut thumb_args = FluidGlassArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::Fixed(thumb_px)),
                Some(DimensionValue::Fixed(thumb_px)),
            ))
            .tint_color(thumb_color)
            .refraction_height(1.0)
            .shape(Shape::Ellipse);
        if let Some(border) = args.thumb_border {
            thumb_args = thumb_args.border(border);
        }
        fluid_glass(&crate::fluid_glass::FluidGlassArgs::with_child(
            &thumb_args,
            || {},
        ));

        // Measurement and placement
        layout(GlassSwitchLayout {
            width: width_px,
            height: height_px,
            thumb_padding: args.thumb_padding.to_px(),
            progress,
        });
    });
}

#[derive(Clone, PartialEq)]
struct GlassSwitchLayout {
    width: Px,
    height: Px,
    thumb_padding: Px,
    progress: f32,
}

impl LayoutSpec for GlassSwitchLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let track_id = input.children_ids()[0];
        let thumb_id = input.children_ids()[1];

        let track_constraint = Constraint::new(
            DimensionValue::Fixed(self.width),
            DimensionValue::Fixed(self.height),
        );
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

        let nodes_constraints = vec![(track_id, track_constraint), (thumb_id, thumb_constraint)];
        let sizes_map = input.measure_children(nodes_constraints)?;

        let thumb_size = sizes_map
            .get(&thumb_id)
            .expect("thumb size should be measured");

        let eased_progress = animation::easing(self.progress);

        output.place_child(track_id, PxPosition::new(Px(0), Px(0)));

        let start_x = self.thumb_padding;
        let end_x = self.width - thumb_size.width - self.thumb_padding;
        let thumb_x = start_x.0 as f32 + (end_x.0 - start_x.0) as f32 * eased_progress;
        let thumb_y = (self.height - thumb_size.height) / 2;

        output.place_child(thumb_id, PxPosition::new(Px(thumb_x as i32), thumb_y));

        Ok(ComputedData {
            width: self.width,
            height: self.height,
        })
    }
}
