//! Modal dialog provider — show modal content above the main app UI.
//!
//! ## Usage
//!
//! Used to show modal dialogs such as alerts, confirmations, wizards and forms; dialogs block interaction with underlying content while active.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp, tessera, winit};

use crate::{
    alignment::Alignment,
    animation,
    boxed::{BoxedArgsBuilder, boxed},
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    pipelines::ShadowProps,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

/// The duration of the full dialog animation.
const ANIM_TIME: Duration = Duration::from_millis(300);

/// Compute normalized (0..1) linear progress from an optional animation timer.
/// Placing this here reduces inline complexity inside the component body.
fn compute_dialog_progress(timer_opt: Option<Instant>) -> f32 {
    timer_opt.as_ref().map_or(1.0, |timer| {
        let elapsed = timer.elapsed();
        if elapsed >= ANIM_TIME {
            1.0
        } else {
            elapsed.as_secs_f32() / ANIM_TIME.as_secs_f32()
        }
    })
}

/// Compute blur radius for glass style scrim.
fn blur_radius_for(progress: f32, is_open: bool, max_blur_radius: f32) -> f32 {
    if is_open {
        progress * max_blur_radius
    } else {
        max_blur_radius * (1.0 - progress)
    }
}

/// Compute scrim alpha for material style.
fn scrim_alpha_for(progress: f32, is_open: bool) -> f32 {
    if is_open {
        progress * 0.5
    } else {
        0.5 * (1.0 - progress)
    }
}

/// Defines the visual style of the dialog's scrim.
#[derive(Default, Clone, Copy)]
pub enum DialogStyle {
    /// A translucent glass effect that blurs the content behind it.
    Glass,
    /// A simple, semi-transparent dark overlay.
    #[default]
    Material,
}

/// Arguments for the [`dialog_provider`] component.
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct DialogProviderArgs {
    /// Callback function triggered when a close request is made, for example by
    /// clicking the scrim or pressing the `ESC` key.
    pub on_close_request: Arc<dyn Fn() + Send + Sync>,
    /// Padding around the dialog content.
    #[builder(default = "Dp(16.0)")]
    pub padding: Dp,
    /// The visual style of the dialog's scrim.
    #[builder(default)]
    pub style: DialogStyle,
}

#[derive(Default)]
struct DialogProviderStateInner {
    is_open: bool,
    timer: Option<Instant>,
}

/// Shared state for [`dialog_provider`], controlling visibility and animation.
///
/// # Example
///
/// ```
/// use tessera_ui_basic_components::dialog::DialogProviderState;
///
/// let state = DialogProviderState::new();
/// assert!(!state.is_open()); // Initially closed
/// state.open();
/// assert!(state.is_open()); // Now opened
/// ```
#[derive(Clone, Default)]
pub struct DialogProviderState {
    inner: Arc<RwLock<DialogProviderStateInner>>,
}

impl DialogProviderState {
    /// Creates a new dialog provider state handle.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the dialog, starting the animation if necessary.
    pub fn open(&self) {
        let mut inner = self.inner.write();
        if !inner.is_open {
            inner.is_open = true;
            let mut timer = Instant::now();
            if let Some(old_timer) = inner.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    timer += ANIM_TIME - elapsed;
                }
            }
            inner.timer = Some(timer);
        }
    }

    /// Closes the dialog, starting the closing animation if necessary.
    pub fn close(&self) {
        let mut inner = self.inner.write();
        if inner.is_open {
            inner.is_open = false;
            let mut timer = Instant::now();
            if let Some(old_timer) = inner.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    timer += ANIM_TIME - elapsed;
                }
            }
            inner.timer = Some(timer);
        }
    }

    /// Returns whether the dialog is currently open.
    pub fn is_open(&self) -> bool {
        self.inner.read().is_open
    }

    /// Returns whether the dialog is mid-animation.
    pub fn is_animating(&self) -> bool {
        self.inner
            .read()
            .timer
            .is_some_and(|t| t.elapsed() < ANIM_TIME)
    }

    fn snapshot(&self) -> (bool, Option<Instant>) {
        let inner = self.inner.read();
        (inner.is_open, inner.timer)
    }
}

fn render_scrim(args: &DialogProviderArgs, is_open: bool, progress: f32) {
    match args.style {
        DialogStyle::Glass => {
            let blur_radius = blur_radius_for(progress, is_open, 5.0);
            fluid_glass(
                FluidGlassArgsBuilder::default()
                    .on_click(args.on_close_request.clone())
                    .tint_color(Color::TRANSPARENT)
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .height(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .dispersion_height(Dp(0.0))
                    .refraction_height(Dp(0.0))
                    .block_input(true)
                    .blur_radius(Dp(blur_radius as f64))
                    .border(None)
                    .shape(Shape::RoundedRectangle {
                        top_left: Dp(0.0),
                        top_right: Dp(0.0),
                        bottom_right: Dp(0.0),
                        bottom_left: Dp(0.0),
                        g2_k_value: 3.0,
                    })
                    .noise_amount(0.0)
                    .build()
                    .expect("builder construction failed"),
                None,
                || {},
            );
        }
        DialogStyle::Material => {
            let alpha = scrim_alpha_for(progress, is_open);
            surface(
                SurfaceArgsBuilder::default()
                    .style(Color::BLACK.with_alpha(alpha).into())
                    .on_click(args.on_close_request.clone())
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .height(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .block_input(true)
                    .build()
                    .expect("builder construction failed"),
                None,
                || {},
            );
        }
    }
}

fn make_keyboard_input_handler(
    on_close: Arc<dyn Fn() + Send + Sync>,
) -> Box<dyn for<'a> Fn(tessera_ui::InputHandlerInput<'a>) + Send + Sync + 'static> {
    Box::new(move |input| {
        input.keyboard_events.drain(..).for_each(|event| {
            if event.state == winit::event::ElementState::Pressed
                && let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) =
                    event.physical_key
            {
                (on_close)();
            }
        });
    })
}

#[tessera]
fn dialog_content_wrapper(
    style: DialogStyle,
    alpha: f32,
    padding: Dp,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    boxed(
        BoxedArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .alignment(Alignment::Center)
            .build()
            .expect("builder construction failed"),
        |scope| {
            scope.child(move || match style {
                DialogStyle::Glass => {
                    fluid_glass(
                        FluidGlassArgsBuilder::default()
                            .tint_color(Color::WHITE.with_alpha(alpha / 2.5))
                            .blur_radius(Dp(5.0 * alpha as f64))
                            .shape(Shape::RoundedRectangle {
                                top_left: Dp(25.0),
                                top_right: Dp(25.0),
                                bottom_right: Dp(25.0),
                                bottom_left: Dp(25.0),
                                g2_k_value: 3.0,
                            })
                            .refraction_amount(32.0 * alpha)
                            .block_input(true)
                            .padding(padding)
                            .build()
                            .expect("builder construction failed"),
                        None,
                        content,
                    );
                }
                DialogStyle::Material => {
                    surface(
                        SurfaceArgsBuilder::default()
                            .style(Color::WHITE.with_alpha(alpha).into())
                            .shadow(ShadowProps {
                                color: Color::BLACK.with_alpha(alpha / 4.0),
                                ..Default::default()
                            })
                            .shape(Shape::RoundedRectangle {
                                top_left: Dp(25.0),
                                top_right: Dp(25.0),
                                bottom_right: Dp(25.0),
                                bottom_left: Dp(25.0),
                                g2_k_value: 3.0,
                            })
                            .padding(padding)
                            .block_input(true)
                            .build()
                            .expect("builder construction failed"),
                        None,
                        content,
                    );
                }
            });
        },
    );
}

/// # dialog_provider
///
/// Provide a modal dialog at the top level of an application.
///
/// ## Usage
///
/// Show modal content for alerts, confirmation dialogs, multi-step forms, or onboarding steps that require blocking user interaction with the main UI.
///
/// ## Parameters
///
/// - `args` — configuration for dialog appearance and the `on_close_request` callback; see [`DialogProviderArgs`].
/// - `state` — a clonable [`DialogProviderState`] handle; use `DialogProviderState::new()` to create one.
/// - `main_content` — closure that renders the always-visible base UI.
/// - `dialog_content` — closure that renders dialog content; receives a `f32` alpha for animation.
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::dialog::DialogProviderState;
/// let state = DialogProviderState::new();
/// assert!(!state.is_open());
/// state.open();
/// assert!(state.is_open());
/// state.close();
/// assert!(!state.is_open());
/// ```
#[tessera]
pub fn dialog_provider(
    args: DialogProviderArgs,
    state: DialogProviderState,
    main_content: impl FnOnce(),
    dialog_content: impl FnOnce(f32) + Send + Sync + 'static,
) {
    // 1. Render the main application content unconditionally.
    main_content();

    // 2. If the dialog is open, render the modal overlay.
    // Sample state once to avoid repeated locks and improve readability.
    let (is_open, timer_opt) = state.snapshot();

    let is_animating = timer_opt.is_some_and(|t| t.elapsed() < ANIM_TIME);

    if is_open || is_animating {
        let progress = animation::easing(compute_dialog_progress(timer_opt));

        let content_alpha = if is_open {
            progress * 1.0 // Transition from 0 to 1 alpha
        } else {
            1.0 * (1.0 - progress) // Transition from 1 to 0 alpha
        };

        // 2a. Scrim (delegated)
        render_scrim(&args, is_open, progress);

        // 2b. Input Handler for intercepting keyboard events (delegated)
        let handler = make_keyboard_input_handler(args.on_close_request.clone());
        input_handler(handler);

        // 2c. Dialog Content
        // The user-defined dialog content is rendered on top of everything.
        dialog_content_wrapper(args.style, content_alpha, args.padding, move || {
            dialog_content(content_alpha);
        });
    }
}
