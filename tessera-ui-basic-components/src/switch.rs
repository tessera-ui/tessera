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
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, PressKeyEventType,
    PxPosition,
    accesskit::{Action, Role, Toggled},
    tessera,
    winit::window::CursorIcon,
};

use crate::{
    animation,
    pipelines::ShapeCommand,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

const ANIMATION_DURATION: Duration = Duration::from_millis(150);

/// Represents the state for the `switch` component, including checked status and animation progress.
///
/// This struct can be shared between multiple switches or managed externally to control the checked state and animation.
pub(crate) struct SwitchStateInner {
    checked: bool,
    progress: f32,
    last_toggle_time: Option<Instant>,
}

impl Default for SwitchStateInner {
    fn default() -> Self {
        Self::new(false)
    }
}

impl SwitchStateInner {
    /// Creates a new `SwitchState` with the given initial checked state.
    pub fn new(initial_state: bool) -> Self {
        Self {
            checked: initial_state,
            progress: if initial_state { 1.0 } else { 0.0 },
            last_toggle_time: None,
        }
    }

    /// Toggles the checked state and updates the animation timestamp.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
        self.last_toggle_time = Some(Instant::now());
    }
}

/// External state handle for the `switch` component.
#[derive(Clone)]
pub struct SwitchState {
    inner: Arc<RwLock<SwitchStateInner>>,
}

impl SwitchState {
    /// Creates a new state handle with the given initial value.
    pub fn new(initial_state: bool) -> Self {
        Self {
            inner: Arc::new(RwLock::new(SwitchStateInner::new(initial_state))),
        }
    }

    pub(crate) fn read(&self) -> RwLockReadGuard<'_, SwitchStateInner> {
        self.inner.read()
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, SwitchStateInner> {
        self.inner.write()
    }

    /// Returns whether the switch is currently checked.
    pub fn is_checked(&self) -> bool {
        self.inner.read().checked
    }

    /// Sets the checked state directly, resetting animation progress.
    pub fn set_checked(&self, checked: bool) {
        let mut inner = self.inner.write();
        if inner.checked != checked {
            inner.checked = checked;
            inner.progress = if checked { 1.0 } else { 0.0 };
            inner.last_toggle_time = None;
        }
    }

    /// Toggles the switch and kicks off the animation timeline.
    pub fn toggle(&self) {
        self.inner.write().toggle();
    }

    /// Returns the current animation progress (0.0..1.0).
    pub fn animation_progress(&self) -> f32 {
        self.inner.read().progress
    }
}

impl Default for SwitchState {
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
    /// Total width of the switch track.
    #[builder(default = "Dp(52.0)")]
    pub width: Dp,
    /// Total height of the switch track (including padding).
    #[builder(default = "Dp(32.0)")]
    pub height: Dp,
    /// Track color when the switch is off.
    #[builder(default = "crate::material_color::global_material_scheme().surface_variant")]
    pub track_color: Color,
    /// Track color when the switch is on.
    #[builder(default = "crate::material_color::global_material_scheme().primary")]
    pub track_checked_color: Color,
    /// Thumb color used in both states.
    #[builder(default = "crate::material_color::global_material_scheme().on_surface")]
    pub thumb_color: Color,
    /// Padding around the thumb inside the track.
    #[builder(default = "Dp(3.0)")]
    pub thumb_padding: Dp,
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

fn update_progress_from_state(state: &SwitchState) {
    let last_toggle_time = state.read().last_toggle_time;
    if let Some(last_toggle_time) = last_toggle_time {
        let elapsed = last_toggle_time.elapsed();
        let fraction = (elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32()).min(1.0);
        let checked = state.read().checked;
        state.write().progress = if checked { fraction } else { 1.0 - fraction };
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
    state: &SwitchState,
    on_toggle: &Option<Arc<dyn Fn(bool) + Send + Sync>>,
    input: &mut tessera_ui::InputHandlerInput,
) {
    update_progress_from_state(state);

    let size = input.computed_data;
    let is_cursor_in = is_cursor_in_component(size, input.cursor_position_rel);

    if is_cursor_in && on_toggle.is_some() {
        input.requests.cursor_icon = CursorIcon::Pointer;
    }

    for e in input.cursor_events.iter() {
        if matches!(
            e.content,
            CursorEventContent::Pressed(PressKeyEventType::Left)
        ) && is_cursor_in
        {
            toggle_switch_state(state, on_toggle);
        }
    }
}

fn toggle_switch_state(
    state: &SwitchState,
    on_toggle: &Option<Arc<dyn Fn(bool) + Send + Sync>>,
) -> bool {
    let Some(on_toggle) = on_toggle else {
        return false;
    };

    state.write().toggle();
    let checked = state.read().checked;
    on_toggle(checked);
    true
}

fn apply_switch_accessibility(
    input: &mut tessera_ui::InputHandlerInput<'_>,
    state: &SwitchState,
    on_toggle: &Option<Arc<dyn Fn(bool) + Send + Sync>>,
    label: Option<&String>,
    description: Option<&String>,
) {
    let checked = state.read().checked;
    let mut builder = input.accessibility().role(Role::Switch);

    if let Some(label) = label {
        builder = builder.label(label.clone());
    }
    if let Some(description) = description {
        builder = builder.description(description.clone());
    }

    builder = builder
        .focusable()
        .action(Action::Click)
        .toggled(if checked {
            Toggled::True
        } else {
            Toggled::False
        });

    builder.commit();

    if on_toggle.is_some() {
        let state = state.clone();
        let on_toggle = on_toggle.clone();
        input.set_accessibility_action_handler(move |action| {
            if action == Action::Click {
                toggle_switch_state(&state, &on_toggle);
            }
        });
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

/// # switch
///
/// Renders an animated on/off toggle switch.
///
/// ## Usage
///
/// Use for settings or any other boolean state that the user can control.
///
/// ## Parameters
///
/// - `args` — configures the switch's appearance and `on_toggle` callback; see [`SwitchArgs`].
/// - `state` — a clonable [`SwitchState`] to manage the checked/unchecked state.
///
/// ## Examples
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui_basic_components::switch::{switch, SwitchArgsBuilder, SwitchState};
///
/// let switch_state = SwitchState::new(false);
///
/// switch(
///     SwitchArgsBuilder::default()
///         .on_toggle(Arc::new(|checked| {
///             println!("Switch is now: {}", if checked { "ON" } else { "OFF" });
///         }))
///         .build()
///         .unwrap(),
///     switch_state,
/// );
/// ```
#[tessera]
pub fn switch(args: impl Into<SwitchArgs>, state: SwitchState) {
    let args: SwitchArgs = args.into();
    let thumb_size = Dp(args.height.0 - (args.thumb_padding.0 * 2.0));

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(thumb_size.to_px()))
            .height(DimensionValue::Fixed(thumb_size.to_px()))
            .style(args.thumb_color.into())
            .shape(Shape::Ellipse)
            .build()
            .expect("builder construction failed"),
        None,
        || {},
    );

    let on_toggle = args.on_toggle.clone();
    let accessibility_on_toggle = on_toggle.clone();
    let accessibility_label = args.accessibility_label.clone();
    let accessibility_description = args.accessibility_description.clone();
    let progress = state.read().progress;

    let state_for_handler = state.clone();
    input_handler(Box::new(move |mut input| {
        // Delegate input handling to the extracted helper.
        handle_input_events_switch(&state_for_handler, &on_toggle, &mut input);
        apply_switch_accessibility(
            &mut input,
            &state_for_handler,
            &accessibility_on_toggle,
            accessibility_label.as_ref(),
            accessibility_description.as_ref(),
        );
    }));

    measure(Box::new(move |input| {
        let thumb_id = input.children_ids[0];
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
        let thumb_size = input.measure_child(thumb_id, &thumb_constraint)?;

        let self_width_px = args.width.to_px();
        let self_height_px = args.height.to_px();
        let thumb_padding_px = args.thumb_padding.to_px();

        let start_x = thumb_padding_px;
        let end_x = self_width_px - thumb_size.width - thumb_padding_px;
        let eased = animation::easing(progress);
        let thumb_x = start_x.0 as f32 + (end_x.0 - start_x.0) as f32 * eased;

        let thumb_y = (self_height_px - thumb_size.height) / 2;

        input.place_child(
            thumb_id,
            PxPosition::new(tessera_ui::Px(thumb_x as i32), thumb_y),
        );

        let track_color = interpolate_color(args.track_color, args.track_checked_color, progress);
        let track_command = ShapeCommand::Rect {
            color: track_color,
            corner_radii: glam::Vec4::splat((self_height_px.0 as f32) / 2.0).into(),
            corner_g2: [2.0; 4], // Use G1 corners here specifically
            shadow: None,
        };
        input.metadata_mut().push_draw_command(track_command);

        Ok(ComputedData {
            width: self_width_px,
            height: self_height_px,
        })
    }));
}
