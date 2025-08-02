#![allow(clippy::needless_pass_by_value)]
//! # Glass Switch Component Module
//!
//! This module provides a customizable, glassmorphic-style switch (toggle) UI component for the Tessera UI framework.
//! The glass switch enables toggling a boolean state with smooth animated transitions and a frosted glass visual effect.
//! It is suitable for modern user interfaces requiring visually appealing, interactive on/off controls, such as settings panels, forms, or dashboards.
//! The component supports extensive customization, including size, color, border, and animation, and is designed for stateless usage with external state management.
//! Typical usage involves integrating the switch into application UIs where a clear, elegant toggle is desired.
//!
//! See [`glass_switch()`](tessera-ui-basic-components/src/glass_switch.rs:142) for usage details and customization options.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::Mutex;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, PressKeyEventType,
    PxPosition, winit::window::CursorIcon,
};
use tessera_ui_macros::tessera;

use crate::{
    animation,
    fluid_glass::{FluidGlassArgsBuilder, GlassBorder, fluid_glass},
    shape_def::Shape,
};

const ANIMATION_DURATION: Duration = Duration::from_millis(150);

/// State for the `glass_switch` component, handling animation.
pub struct GlassSwitchState {
    pub checked: bool,
    progress: Mutex<f32>,
    last_toggle_time: Mutex<Option<Instant>>,
}

impl GlassSwitchState {
    pub fn new(initial_state: bool) -> Self {
        Self {
            checked: initial_state,
            progress: Mutex::new(if initial_state { 1.0 } else { 0.0 }),
            last_toggle_time: Mutex::new(None),
        }
    }

    pub fn toggle(&mut self) {
        self.checked = !self.checked;
        *self.last_toggle_time.lock() = Some(Instant::now());
    }
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct GlassSwitchArgs {
    #[builder(default)]
    pub state: Option<Arc<Mutex<GlassSwitchState>>>,

    #[builder(default = "false")]
    pub checked: bool,

    #[builder(default = "Arc::new(|_| {})")]
    pub on_toggle: Arc<dyn Fn(bool) + Send + Sync>,

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
/// use tessera_ui_basic_components::glass_switch::{glass_switch, GlassSwitchArgs, GlassSwitchArgsBuilder};
///
/// // In a real app, you would manage the state.
/// // This example shows how to create a switch that is initially off.
/// glass_switch(
///     GlassSwitchArgsBuilder::default()
///         .checked(false)
///         .on_toggle(Arc::new(|new_state| {
///             // Update your application state here
///             println!("Switch toggled to: {}", new_state);
///         }))
///         .build()
///         .unwrap(),
/// );
///
/// // An initially checked switch
/// glass_switch(
///     GlassSwitchArgsBuilder::default()
///         .checked(true)
///         .build()
///         .unwrap(),
/// );
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
pub fn glass_switch(args: impl Into<GlassSwitchArgs>) {
    let args: GlassSwitchArgs = args.into();
    let thumb_size = Dp(args.height.0 - (args.thumb_padding.0 * 2.0));

    // Track (background) as the first child, rendered with fluid_glass
    let progress = args
        .state
        .as_ref()
        .map(|s| *s.lock().progress.lock())
        .unwrap_or(if args.checked { 1.0 } else { 0.0 });
    let track_color = Color {
        r: args.track_off_color.r + (args.track_on_color.r - args.track_off_color.r) * progress,
        g: args.track_off_color.g + (args.track_on_color.g - args.track_off_color.g) * progress,
        b: args.track_off_color.b + (args.track_on_color.b - args.track_off_color.b) * progress,
        a: args.track_off_color.a + (args.track_on_color.a - args.track_off_color.a) * progress,
    };
    let mut arg = FluidGlassArgsBuilder::default()
        .width(DimensionValue::Fixed(args.width.to_px()))
        .height(DimensionValue::Fixed(args.height.to_px()))
        .tint_color(track_color)
        .blur_radius(10.0)
        .shape(Shape::RoundedRectangle {
            corner_radius: args.height.to_px().to_f32() / 2.0,
            g2_k_value: 2.0,
        })
        .blur_radius(8.0);
    if let Some(border) = args.track_border {
        arg = arg.border(border);
    }
    let track_glass_arg = arg.build().unwrap();
    fluid_glass(track_glass_arg, None, || {});

    // Thumb (slider) is always white, opacity changes with progress
    let thumb_alpha =
        args.thumb_off_alpha + (args.thumb_on_alpha - args.thumb_off_alpha) * progress;
    let thumb_color = Color::new(1.0, 1.0, 1.0, thumb_alpha);
    let mut thumb_glass_arg = FluidGlassArgsBuilder::default()
        .width(DimensionValue::Fixed(thumb_size.to_px()))
        .height(DimensionValue::Fixed(thumb_size.to_px()))
        .tint_color(thumb_color)
        .refraction_height(1.0)
        .shape(Shape::Ellipse);
    if let Some(border) = args.thumb_border {
        thumb_glass_arg = thumb_glass_arg.border(border);
    }
    let thumb_glass_arg = thumb_glass_arg.build().unwrap();
    fluid_glass(thumb_glass_arg, None, || {});

    let on_toggle = args.on_toggle.clone();
    let state = args.state.clone();
    let checked = args.checked;

    state_handler(Box::new(move |input| {
        if let Some(state) = &state {
            let state = state.lock();
            let mut progress = state.progress.lock();
            if let Some(last_toggle_time) = *state.last_toggle_time.lock() {
                let elapsed = last_toggle_time.elapsed();
                let animation_fraction =
                    (elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32()).min(1.0);
                *progress = if state.checked {
                    animation_fraction
                } else {
                    1.0 - animation_fraction
                };
            }
        }

        let size = input.computed_data;
        let is_cursor_in = if let Some(pos) = input.cursor_position_rel {
            pos.x.0 >= 0 && pos.x.0 < size.width.0 && pos.y.0 >= 0 && pos.y.0 < size.height.0
        } else {
            false
        };
        if is_cursor_in {
            input.requests.cursor_icon = CursorIcon::Pointer;
        }
        for e in input.cursor_events.iter() {
            if let CursorEventContent::Pressed(PressKeyEventType::Left) = &e.content {
                if is_cursor_in {
                    if let Some(state) = &state {
                        state.lock().toggle();
                    }
                    on_toggle(!checked);
                }
            }
        }
    }));

    measure(Box::new(move |input| {
        let track_id = input.children_ids[0]; // track is the first child
        let thumb_id = input.children_ids[1]; // thumb is the second child
        // Prepare constraints for both children
        let track_constraint = Constraint::new(
            DimensionValue::Fixed(args.width.to_px()),
            DimensionValue::Fixed(args.height.to_px()),
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
        // Measure both children in parallel
        let nodes_constraints = vec![(track_id, track_constraint), (thumb_id, thumb_constraint)];
        let sizes_map = input.measure_children(nodes_constraints)?;

        let _track_size = sizes_map.get(&track_id).unwrap();
        let thumb_size = sizes_map.get(&thumb_id).unwrap();
        let self_width_px = args.width.to_px();
        let self_height_px = args.height.to_px();
        let thumb_padding_px = args.thumb_padding.to_px();

        let progress = animation::easing(
            args.state
                .as_ref()
                .map(|s| *s.lock().progress.lock())
                .unwrap_or(if args.checked { 1.0 } else { 0.0 }),
        );
        // Place track at origin
        input.place_child(
            track_id,
            PxPosition::new(tessera_ui::Px(0), tessera_ui::Px(0)),
        );
        // Place thumb according to progress
        let start_x = thumb_padding_px;
        let end_x = self_width_px - thumb_size.width - thumb_padding_px;
        let thumb_x = start_x.0 as f32 + (end_x.0 - start_x.0) as f32 * progress;
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
