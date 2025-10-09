//! A modal dialog component for displaying critical information or actions.
//!
//! This module provides [`dialog_provider`], a component that renders content in a modal
//! overlay. When active, the dialog sits on top of the primary UI, blocks interactions
//! with the content behind it (via a "scrim"), and can be dismissed by user actions
//! like pressing the `Escape` key or clicking the scrim.
//!
//! # Key Components
//!
//! * **[`dialog_provider`]**: The main function that wraps your UI to provide dialog capabilities.
//! * **[`DialogProviderState`]**: A state object you create and manage to control the
//!   dialog's visibility using its [`open()`](DialogProviderState::open) and
//!   [`close()`](DialogProviderState::close) methods.
//! * **[`DialogProviderArgs`]**: Configuration for the provider, including the visual
//!   [`style`](DialogStyle) of the scrim and the mandatory `on_close_request` callback.
//! * **[`DialogStyle`]**: Defines the scrim's appearance, either `Material` (a simple dark
//!   overlay) or `Glass` (a blurred, translucent effect).
//!
//! # Usage
//!
//! The `dialog_provider` acts as a wrapper around your main content. It takes the main
//! content and the dialog content as separate closures.
//!
//! 1.  **Create State**: In your application's state, create an `Arc<RwLock<DialogProviderState>>`.
//! 2.  **Wrap Content**: Call `dialog_provider` at a high level in your component tree.
//! 3.  **Provide Content**: Pass two closures to `dialog_provider`:
//!     - `main_content`: Renders the UI that is always visible.
//!     - `dialog_content`: Renders the content of the dialog box itself. This closure
//!       receives an `f32` alpha value for animating its appearance.
//! 4.  **Control Visibility**: From an event handler (e.g., a button's `on_click`), call
//!     `dialog_state.write().open()` to show the dialog.
//! 5.  **Handle Closing**: The `on_close_request` callback you provide is responsible for
//!     calling `dialog_state.write().close()` to dismiss the dialog.
//!
//! # Example
//!
//! ```
//! use std::sync::Arc;
//! use parking_lot::RwLock;
//! use tessera_ui::{tessera, Renderer};
//! use tessera_ui_basic_components::{
//!     dialog::{dialog_provider, DialogProviderArgsBuilder, DialogProviderState},
//!     button::{button, ButtonArgsBuilder},
//!     ripple_state::RippleState,
//!     text::{text, TextArgsBuilder},
//! };
//!
//! // Define an application state.
//! #[derive(Default)]
//! struct AppState {
//!     dialog_state: Arc<RwLock<DialogProviderState>>,
//!     ripple_state: Arc<RippleState>,
//! }
//!
//! #[tessera]
//! fn app(state: Arc<RwLock<AppState>>) {
//!     let dialog_state = state.read().dialog_state.clone();
//!
//!     // Use the dialog_provider.
//!     dialog_provider(
//!         DialogProviderArgsBuilder::default()
//!             // Provide a callback to handle close requests.
//!             .on_close_request(Arc::new({
//!                 let dialog_state = dialog_state.clone();
//!                 move || dialog_state.write().close()
//!             }))
//!             .build()
//!             .unwrap(),
//!         dialog_state.clone(),
//!         // Define the main content.
//!         move || {
//!             button(
//!                 ButtonArgsBuilder::default()
//!                     .on_click(Arc::new({
//!                         let dialog_state = dialog_state.clone();
//!                         move || dialog_state.write().open()
//!                     }))
//!                     .build()
//!                     .unwrap(),
//!                 state.read().ripple_state.clone(),
//!                 || text(TextArgsBuilder::default().text("Show Dialog".to_string()).build().unwrap())
//!             );
//!         },
//!         // Define the dialog content.
//!         |alpha| {
//!             text(TextArgsBuilder::default().text("This is a dialog!".to_string()).build().unwrap());
//!         }
//!     );
//! }
//! ```
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
pub struct DialogProviderState {
    is_open: bool,
    timer: Option<Instant>,
}

impl DialogProviderState {
    /// Open the dialog
    pub fn open(&mut self) {
        if self.is_open {
            // Already opened, no action needed
        } else {
            self.is_open = true; // Mark as open
            let mut timer = Instant::now();
            if let Some(old_timer) = self.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    // If we are still in the middle of an animation
                    timer += ANIM_TIME - elapsed; // We need to 'catch up' the timer
                }
            }
            self.timer = Some(timer);
        }
    }

    /// Close the dialog
    pub fn close(&mut self) {
        if self.is_open {
            self.is_open = false; // Mark as closed
            let mut timer = Instant::now();
            if let Some(old_timer) = self.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    // If we are still in the middle of an animation
                    timer += ANIM_TIME - elapsed; // We need to 'catch up' the timer
                }
            }
            self.timer = Some(timer);
        }
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
                    .dispersion_height(0.0)
                    .refraction_height(0.0)
                    .block_input(true)
                    .blur_radius(blur_radius)
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
                    .unwrap(),
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
                    .unwrap(),
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
            .unwrap(),
        |scope| {
            scope.child(move || match style {
                DialogStyle::Glass => {
                    fluid_glass(
                        FluidGlassArgsBuilder::default()
                            .tint_color(Color::WHITE.with_alpha(alpha / 2.5))
                            .blur_radius(5.0 * alpha)
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
                            .unwrap(),
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
                            .unwrap(),
                        None,
                        content,
                    );
                }
            });
        },
    );
}

/// A provider component that manages the rendering and event flow for a modal dialog.
///
/// This component should be used as one of the outermost layers of the application.
/// It renders the main content, and when `is_open` is true, it overlays a modal
/// dialog, intercepting all input events to create a modal experience.
///
/// The dialog can be closed by calling the `on_close_request` callback, which can be
/// triggered by clicking the background scrim or pressing the `ESC` key.
///
/// # Arguments
///
/// - `args` - The arguments for configuring the dialog provider. See [`DialogProviderArgs`].
/// - `main_content` - A closure that renders the main content of the application,
///   which is visible whether the dialog is open or closed.
/// - `dialog_content` - A closure that renders the content of the dialog, which is
///   only visible when `args.is_open` is `true`.
#[tessera]
pub fn dialog_provider(
    args: DialogProviderArgs,
    state: Arc<RwLock<DialogProviderState>>,
    main_content: impl FnOnce(),
    dialog_content: impl FnOnce(f32) + Send + Sync + 'static,
) {
    // 1. Render the main application content unconditionally.
    main_content();

    // 2. If the dialog is open, render the modal overlay.
    // Sample state once to avoid repeated locks and improve readability.
    let (is_open, timer_opt) = {
        let guard = state.read();
        (guard.is_open, guard.timer)
    };

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
