use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp};
use tessera_ui_macros::tessera;

use crate::{
    alignment::Alignment,
    boxed::{BoxedArgs, boxed_ui},
    checkmark::{CheckmarkArgsBuilder, checkmark},
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

#[derive(Clone)]
pub struct CheckboxState {
    pub ripple: Arc<crate::ripple_state::RippleState>,
    pub checkmark: Arc<RwLock<CheckmarkState>>,
}

impl CheckboxState {
    pub fn new(checked: bool) -> Self {
        Self {
            ripple: Arc::new(crate::ripple_state::RippleState::new()),
            checkmark: Arc::new(RwLock::new(CheckmarkState::new(checked))),
        }
    }
}

/// Arguments for the `checkbox` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct CheckboxArgs {
    #[builder(default)]
    pub checked: bool,

    #[builder(default = "Arc::new(|_| {})")]
    pub on_toggle: Arc<dyn Fn(bool) + Send + Sync>,

    #[builder(default = "Dp(24.0)")]
    pub size: Dp,

    #[builder(default = "Color::new(0.8, 0.8, 0.8, 1.0)")]
    pub color: Color,

    #[builder(default = "Color::new(0.6, 0.7, 0.9, 1.0)")]
    pub checked_color: Color,

    #[builder(default = "Color::from_rgb_u8(119, 72, 146)")]
    pub checkmark_color: Color,

    #[builder(default = "5.0")]
    pub checkmark_stroke_width: f32,

    #[builder(default = "1.0")]
    pub checkmark_animation_progress: f32,

    #[builder(default = "Shape::RoundedRectangle{ corner_radius: 4.0 }")]
    pub shape: Shape,

    #[builder(default)]
    pub hover_color: Option<Color>,

    #[builder(default = "None")]
    pub state: Option<Arc<CheckboxState>>,
}

impl Default for CheckboxArgs {
    fn default() -> Self {
        CheckboxArgsBuilder::default().build().unwrap()
    }
}

// Animation duration for the checkmark stroke (milliseconds)
const CHECKMARK_ANIMATION_DURATION: Duration = Duration::from_millis(200);

/// State for checkmark animation (similar风格 to `SwitchState`)
pub struct CheckmarkState {
    pub checked: bool,
    progress: f32,
    last_toggle_time: Option<Instant>,
}

impl CheckmarkState {
    pub fn new(initial_state: bool) -> Self {
        Self {
            checked: initial_state,
            progress: if initial_state { 1.0 } else { 0.0 },
            last_toggle_time: None,
        }
    }

    /// Toggle checked state and start animation
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
        self.last_toggle_time = Some(Instant::now());
    }

    /// Update progress based on elapsed time
    pub fn update_progress(&mut self) {
        if let Some(start) = self.last_toggle_time {
            let elapsed = start.elapsed();
            let fraction =
                (elapsed.as_secs_f32() / CHECKMARK_ANIMATION_DURATION.as_secs_f32()).min(1.0);
            self.progress = if self.checked {
                fraction
            } else {
                1.0 - fraction
            };
            if fraction >= 1.0 {
                self.last_toggle_time = None; // Animation ends
            }
        }
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }
}

#[tessera]
pub fn checkbox(args: impl Into<CheckboxArgs>) {
    let args: CheckboxArgs = args.into();

    // Optional external animation state, similar to Switch component pattern
    let state = args.state.clone();

    // If a state is provided, set up an updater to advance the animation each frame
    if let Some(state_for_handler) = state.clone() {
        let checkmark_state = state_for_handler.checkmark.clone();
        state_handler(Box::new(move |_input| {
            checkmark_state.write().update_progress();
        }));
    }

    // Click handler: toggle animation state if present, otherwise simply forward toggle callback
    let on_click = {
        let state = state.clone();
        let on_toggle = args.on_toggle.clone();
        let checked_initial = args.checked;
        Arc::new(move || {
            if let Some(state) = &state {
                state.checkmark.write().toggle();
                on_toggle(state.checkmark.read().checked);
            } else {
                // Fallback: no internal animation state, just invert checked value
                on_toggle(!checked_initial);
            }
        })
    };

    let ripple_state = state.as_ref().map(|s| s.ripple.clone());

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(args.size.to_px()))
            .height(DimensionValue::Fixed(args.size.to_px()))
            .color(if args.checked {
                args.checked_color
            } else {
                args.color
            })
            .hover_color(args.hover_color)
            .shape(args.shape)
            .on_click(Some(on_click))
            .build()
            .unwrap(),
        ripple_state,
        {
            let state_for_child = state.clone();
            move || {
                let progress = state_for_child
                    .as_ref()
                    .map(|s| s.checkmark.read().progress())
                    .unwrap_or(if args.checked { 1.0 } else { 0.0 });
                if progress > 0.0 {
                    surface(
                        SurfaceArgsBuilder::default()
                            .padding(Dp(2.0))
                            .color(Color::TRANSPARENT)
                            .build()
                            .unwrap(),
                        None,
                        move || {
                            boxed_ui!(
                                BoxedArgs {
                                    alignment: Alignment::Center,
                                    ..Default::default()
                                },
                                move || checkmark(
                                    CheckmarkArgsBuilder::default()
                                        .color(args.checkmark_color)
                                        .stroke_width(args.checkmark_stroke_width)
                                        .progress(progress)
                                        .size(Dp(args.size.0 * 0.7))
                                        .padding([5.0, 5.0])
                                        .build()
                                        .unwrap()
                                )
                            );
                        },
                    )
                }
            }
        },
    );
}
