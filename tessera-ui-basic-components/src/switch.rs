#![allow(clippy::needless_return)]
//! # Switch Component Module
//!
//! This module provides a customizable toggle switch UI component for boolean state management in the Tessera UI framework.
//! The `switch` component is commonly used for toggling settings or preferences in user interfaces, offering a modern,
//! animated on/off control. It supports both controlled (external state via [`SwitchState`]) and uncontrolled usage
//! (via `checked` and `on_toggle` parameters), and allows for appearance customization such as track and thumb colors, size, and padding.
//!
//! ## Typical Usage
//! - Settings panels, feature toggles, or any scenario requiring a boolean on/off control.
//! - Can be integrated into forms or interactive UIs where immediate feedback and smooth animation are desired.
//!
//! ## Key Features
//! - Stateless component model: state is managed externally or via parameters, following Tessera's architecture.
//! - Animation support for smooth transitions between checked and unchecked states.
//! - Highly customizable appearance and behavior via [`SwitchArgs`].
//! - Designed for ergonomic integration with the Tessera component tree and event system.
//!
//! See [`SwitchArgs`], [`SwitchState`], and [`switch()`] for details and usage examples.

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
    pipelines::ShapeCommand,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

const ANIMATION_DURATION: Duration = Duration::from_millis(150);

///
/// Represents the state for the `switch` component, including checked status and animation progress.
///
/// This struct can be shared between multiple switches or managed externally to control the checked state and animation.
///
/// # Fields
/// - `checked`: Indicates whether the switch is currently on (`true`) or off (`false`).
///
/// # Example
/// ```
/// use tessera_ui_basic_components::switch::{SwitchState, SwitchArgs, switch};
/// use std::sync::{Arc};
/// use parking_lot::Mutex;
///
/// let state = Arc::new(Mutex::new(SwitchState::new(false)));
///
/// switch(SwitchArgs {
///     state: Some(state.clone()),
///     on_toggle: Arc::new(move |checked| {
///         state.lock().checked = checked;
///     }),
///     ..Default::default()
/// });
/// ```
pub struct SwitchState {
    pub checked: bool,
    progress: Mutex<f32>,
    last_toggle_time: Mutex<Option<Instant>>,
}

impl SwitchState {
    /// Creates a new `SwitchState` with the given initial checked state.
    ///
    /// # Arguments
    /// * `initial_state` - Whether the switch should start as checked (`true`) or unchecked (`false`).
    pub fn new(initial_state: bool) -> Self {
        Self {
            checked: initial_state,
            progress: Mutex::new(if initial_state { 1.0 } else { 0.0 }),
            last_toggle_time: Mutex::new(None),
        }
    }

    /// Toggles the checked state and updates the animation timestamp.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
        *self.last_toggle_time.lock() = Some(Instant::now());
    }
}

///
/// Arguments for configuring the `switch` component.
///
/// This struct allows customization of the switch's state, appearance, and behavior.
///
/// # Fields
/// - `state`: Optional external state for the switch. If provided, the switch will use and update this state.
/// - `checked`: Initial checked state if `state` is not provided.
/// - `on_toggle`: Callback invoked when the switch is toggled, receiving the new checked state.
/// - `width`: Width of the switch track.
/// - `height`: Height of the switch track.
/// - `track_color`: Color of the track when unchecked.
/// - `track_checked_color`: Color of the track when checked.
/// - `thumb_color`: Color of the thumb (handle).
/// - `thumb_padding`: Padding between the thumb and the track edge.
///
/// # Example
/// ```
/// use tessera_ui_basic_components::switch::{SwitchArgs, switch};
/// use std::sync::Arc;
///
/// switch(SwitchArgs {
///     checked: true,
///     on_toggle: Arc::new(|checked| {
///         println!("Switch toggled: {}", checked);
///     }),
///     ..Default::default()
/// });
/// ```
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct SwitchArgs {
    #[builder(default)]
    pub state: Option<Arc<Mutex<SwitchState>>>,

    #[builder(default = "false")]
    pub checked: bool,

    #[builder(default = "Arc::new(|_| {})")]
    pub on_toggle: Arc<dyn Fn(bool) + Send + Sync>,

    #[builder(default = "Dp(52.0)")]
    pub width: Dp,

    #[builder(default = "Dp(32.0)")]
    pub height: Dp,

    #[builder(default = "Color::new(0.8, 0.8, 0.8, 1.0)")]
    pub track_color: Color,

    #[builder(default = "Color::new(0.6, 0.7, 0.9, 1.0)")]
    pub track_checked_color: Color,

    #[builder(default = "Color::WHITE")]
    pub thumb_color: Color,

    #[builder(default = "Dp(3.0)")]
    pub thumb_padding: Dp,
}

impl Default for SwitchArgs {
    fn default() -> Self {
        SwitchArgsBuilder::default().build().unwrap()
    }
}

///
/// A UI component that displays a toggle switch for boolean state.
///
/// The `switch` component provides a customizable on/off control, commonly used for toggling settings.
/// It can be controlled via external state (`SwitchState`) or by using the `checked` and `on_toggle` parameters.
///
/// # Arguments
/// * `args` - Parameters for configuring the switch, see [`SwitchArgs`](crate::switch::SwitchArgs).
///
/// # Example
/// ```
/// use tessera_ui_basic_components::switch::{SwitchArgs, switch};
/// use std::sync::Arc;
///
/// switch(SwitchArgs {
///     checked: false,
///     on_toggle: Arc::new(|checked| {
///         println!("Switch toggled: {}", checked);
///     }),
///     width: tessera_ui::Dp(60.0),
///     height: tessera_ui::Dp(36.0),
///     ..Default::default()
/// });
/// ```
#[tessera]
pub fn switch(args: impl Into<SwitchArgs>) {
    let args: SwitchArgs = args.into();
    let thumb_size = Dp(args.height.0 - (args.thumb_padding.0 * 2.0));

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(thumb_size.to_px()))
            .height(DimensionValue::Fixed(thumb_size.to_px()))
            .color(args.thumb_color)
            .shape(Shape::Ellipse)
            .build()
            .unwrap(),
        None,
        || {},
    );

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
                    on_toggle(!checked);
                }
            }
        }
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

        let progress = args
            .state
            .as_ref()
            .map(|s| *s.lock().progress.lock())
            .unwrap_or(if args.checked { 1.0 } else { 0.0 });

        let start_x = thumb_padding_px;
        let end_x = self_width_px - thumb_size.width - thumb_padding_px;
        let thumb_x = start_x.0 as f32 + (end_x.0 - start_x.0) as f32 * progress;

        let thumb_y = (self_height_px - thumb_size.height) / 2;

        input.place_child(
            thumb_id,
            PxPosition::new(tessera_ui::Px(thumb_x as i32), thumb_y),
        );

        let track_color = if args.checked {
            args.track_checked_color
        } else {
            args.track_color
        };
        let track_command = ShapeCommand::Rect {
            color: track_color,
            corner_radius: (self_height_px.0 as f32) / 2.0,
            g2_k_value: 2.0, // Use G1 corners here specifically
            shadow: None,
        };
        input.metadata_mut().push_draw_command(track_command);

        Ok(ComputedData {
            width: self_width_px,
            height: self_height_px,
        })
    }));
}
