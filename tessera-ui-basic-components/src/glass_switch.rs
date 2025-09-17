//! # Glass Switch Component Module
//!
//! This module provides a customizable, glassmorphic-style switch (toggle) UI component for the Tessera UI framework.
//! The glass switch enables toggling a boolean state with smooth animated transitions and a frosted glass visual effect.
//! It is suitable for modern user interfaces requiring visually appealing, interactive on/off controls, such as settings panels, forms, or dashboards.
//! The component supports extensive customization, including size, color, border, and animation, and is designed for stateless usage with external state management.
//! Typical usage involves integrating the switch into application UIs where a clear, elegant toggle is desired.
//!
//! See [`glass_switch`] for usage details and customization options.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, PressKeyEventType,
    PxPosition, tessera, winit::window::CursorIcon,
};

use crate::{
    animation,
    fluid_glass::{FluidGlassArgsBuilder, GlassBorder, fluid_glass},
    shape_def::Shape,
};

const ANIMATION_DURATION: Duration = Duration::from_millis(150);

/// State for the `glass_switch` component, handling animation.
pub struct GlassSwitchState {
    checked: bool,
    progress: f32,
    last_toggle_time: Option<Instant>,
}

impl Default for GlassSwitchState {
    fn default() -> Self {
        Self::new(false)
    }
}

impl GlassSwitchState {
    /// Creates a new `GlassSwitchState` with the given initial checked state.
    pub fn new(initial_state: bool) -> Self {
        Self {
            checked: initial_state,
            progress: if initial_state { 1.0 } else { 0.0 },
            last_toggle_time: None,
        }
    }

    /// Toggles the switch state.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
        self.last_toggle_time = Some(Instant::now());
    }

    /// Returns whether the switch is currently checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct GlassSwitchArgs {
    #[builder(default, setter(strip_option))]
    pub on_toggle: Option<Arc<dyn Fn(bool) + Send + Sync>>,

    #[builder(default = "Dp(52.0)")]
    pub width: Dp,

    #[builder(default = "Dp(32.0)")]
    pub height: Dp,

    /// Track color when switch is ON
    #[builder(default = "Color::new(0.2, 0.7, 1.0, 0.5)")]
    pub track_on_color: Color,
    /// Track color when switch is OFF
    #[builder(default = "Color::new(0.8, 0.8, 0.8, 0.5)")]
    pub track_off_color: Color,

    /// Thumb alpha when switch is ON (opacity when ON)
    #[builder(default = "0.5")]
    pub thumb_on_alpha: f32,
    /// Thumb alpha when switch is OFF (opacity when OFF)
    #[builder(default = "1.0")]
    pub thumb_off_alpha: f32,

    /// Border for the thumb
    #[builder(default, setter(strip_option))]
    pub thumb_border: Option<GlassBorder>,

    /// Border for the track
    #[builder(default, setter(strip_option))]
    pub track_border: Option<GlassBorder>,

    /// Padding around the thumb
    #[builder(default = "Dp(3.0)")]
    pub thumb_padding: Dp,
}

impl Default for GlassSwitchArgs {
    fn default() -> Self {
        GlassSwitchArgsBuilder::default().build().unwrap()
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

fn update_progress_from_state(state: Arc<RwLock<GlassSwitchState>>) {
    let last_toggle_time = state.read().last_toggle_time;
    if let Some(last_toggle_time) = last_toggle_time {
        let elapsed = last_toggle_time.elapsed();
        let fraction = (elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32()).min(1.0);
        let checked = state.read().checked;
        state.write().progress = if checked { fraction } else { 1.0 - fraction };
    }
}

/// Return true if the given cursor position is inside the component bounds.
fn is_cursor_inside(size: ComputedData, cursor_pos: Option<PxPosition>) -> bool {
    cursor_pos
        .map(|pos| {
            pos.x.0 >= 0 && pos.x.0 < size.width.0 && pos.y.0 >= 0 && pos.y.0 < size.height.0
        })
        .unwrap_or(false)
}

/// Return true if there is a left-press event in the input.
fn was_pressed_left(input: &tessera_ui::InputHandlerInput) -> bool {
    input.cursor_events.iter().any(|e| {
        matches!(
            e.content,
            CursorEventContent::Pressed(PressKeyEventType::Left)
        )
    })
}

fn handle_input_events(
    state: Arc<RwLock<GlassSwitchState>>,
    on_toggle: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    input: &mut tessera_ui::InputHandlerInput,
) {
    let Some(on_toggle) = on_toggle else {
        // No callback provided, do nothing (act disabled)
        return;
    };

    // Update progress first
    update_progress_from_state(state.clone());

    // Cursor handling
    let size = input.computed_data;
    let is_cursor_in = is_cursor_inside(size, input.cursor_position_rel);

    if is_cursor_in {
        input.requests.cursor_icon = CursorIcon::Pointer;
    }

    // Handle press events: toggle state and call callback
    let pressed = was_pressed_left(&*input);

    if pressed && is_cursor_in {
        // If internal state exists, toggle it and use the toggled value.
        state.write().toggle();
        on_toggle(state.read().checked);
    }
}
#[tessera]
/// A glass-like switch component for toggling a boolean state.
///
/// The `glass_switch` provides a visually appealing switch with a frosted glass effect.
/// It animates smoothly between its "on" and "off" states and is fully customizable
/// in terms of size, color, and border.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui_basic_components::glass_switch::{glass_switch, GlassSwitchArgs, GlassSwitchArgsBuilder, GlassSwitchState};
/// use parking_lot::RwLock;
///
/// // In a real app, you would manage the state in shard state or elsewhere.
/// let state = Arc::new(RwLock::new(GlassSwitchState::new(false)));
///
/// glass_switch(
///     GlassSwitchArgsBuilder::default()
///         .on_toggle(Arc::new(|new_state| {
///             // Update your application state here
///             println!("Switch toggled to: {}", new_state);
///         }))
///         .build()
///         .unwrap(),
///     state.clone()
/// );
///
/// // Use the state to toggle the switch programmatically if needed.
/// state.write().toggle(); // Toggle the switch state
/// // or get the current on/off state
/// assert_eq!(state.read().is_checked(), true); // true here after toggle
/// ```
///
/// # Arguments
///
/// * `args` - An instance of `GlassSwitchArgs` which can be built using `GlassSwitchArgsBuilder`.
///   - `checked`: A `bool` indicating the current state of the switch (`true` for on, `false` for off).
///   - `on_toggle`: A callback `Arc<dyn Fn(bool) + Send + Sync>` that is called when the switch is clicked.
///     It receives the new boolean state.
///   - Other arguments for customization like `width`, `height`, `track_on_color`, `track_off_color`, etc.
///     are also available.
pub fn glass_switch(args: impl Into<GlassSwitchArgs>, state: Arc<RwLock<GlassSwitchState>>) {
    let args: GlassSwitchArgs = args.into();
    // Precompute pixel sizes to avoid repeated conversions
    let width_px = args.width.to_px();
    let height_px = args.height.to_px();
    let thumb_dp = Dp(args.height.0 - (args.thumb_padding.0 * 2.0));
    let thumb_px = thumb_dp.to_px();
    let track_radius_dp = Dp(args.height.0 / 2.0);

    // Track tint color interpolation based on progress
    let progress = state.read().progress;
    let track_color = interpolate_color(args.track_off_color, args.track_on_color, progress);

    // Build and render track
    let mut track_builder = FluidGlassArgsBuilder::default()
        .width(DimensionValue::Fixed(width_px))
        .height(DimensionValue::Fixed(height_px))
        .tint_color(track_color)
        .shape({
            Shape::RoundedRectangle {
                top_left: track_radius_dp,
                top_right: track_radius_dp,
                bottom_right: track_radius_dp,
                bottom_left: track_radius_dp,
                g2_k_value: 2.0, // Capsule shape
            }
        })
        .blur_radius(8.0);
    if let Some(border) = args.track_border {
        track_builder = track_builder.border(border);
    }
    fluid_glass(track_builder.build().unwrap(), None, || {});

    // Build and render thumb
    let thumb_alpha =
        args.thumb_off_alpha + (args.thumb_on_alpha - args.thumb_off_alpha) * progress;
    let thumb_color = Color::new(1.0, 1.0, 1.0, thumb_alpha);
    let mut thumb_builder = FluidGlassArgsBuilder::default()
        .width(DimensionValue::Fixed(thumb_px))
        .height(DimensionValue::Fixed(thumb_px))
        .tint_color(thumb_color)
        .refraction_height(1.0)
        .shape(Shape::Ellipse);
    if let Some(border) = args.thumb_border {
        thumb_builder = thumb_builder.border(border);
    }
    fluid_glass(thumb_builder.build().unwrap(), None, || {});

    let state_clone = state.clone();
    let on_toggle = args.on_toggle.clone();
    input_handler(Box::new(move |mut input| {
        handle_input_events(state_clone.clone(), on_toggle.clone(), &mut input);
    }));

    // Measurement and placement
    measure(Box::new(move |input| {
        // Expect track then thumb as children
        let track_id = input.children_ids[0];
        let thumb_id = input.children_ids[1];

        let track_constraint = Constraint::new(
            DimensionValue::Fixed(width_px),
            DimensionValue::Fixed(height_px),
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

        // Measure both children
        let nodes_constraints = vec![(track_id, track_constraint), (thumb_id, thumb_constraint)];
        let sizes_map = input.measure_children(nodes_constraints)?;

        let _track_size = sizes_map.get(&track_id).unwrap();
        let thumb_size = sizes_map.get(&thumb_id).unwrap();
        let self_width_px = width_px;
        let self_height_px = height_px;
        let thumb_padding_px = args.thumb_padding.to_px();

        // Use eased progress for placement
        let eased_progress = animation::easing(state.read().progress);

        input.place_child(
            track_id,
            PxPosition::new(tessera_ui::Px(0), tessera_ui::Px(0)),
        );

        let start_x = thumb_padding_px;
        let end_x = self_width_px - thumb_size.width - thumb_padding_px;
        let thumb_x = start_x.0 as f32 + (end_x.0 - start_x.0) as f32 * eased_progress;
        let thumb_y = (self_height_px - thumb_size.height) / 2;

        input.place_child(
            thumb_id,
            PxPosition::new(tessera_ui::Px(thumb_x as i32), thumb_y),
        );

        Ok(ComputedData {
            width: self_width_px,
            height: self_height_px,
        })
    }));
}
