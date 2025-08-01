//! Provides a modal glass dialog component for overlaying content and intercepting user input.
//!
//! This module defines a dialog provider for creating modal glass dialogs in UI applications.
//! It allows rendering custom dialog content above the main application, blocking interaction
//! with underlying elements and intercepting keyboard/mouse events (such as ESC or scrim click)
//! to trigger close actions. Typical use cases include confirmation dialogs, alerts, and
//! any scenario requiring user attention before proceeding.
//!
//! The dialog is managed via [`GlassDialogProviderArgs`] and the [`glass_dialog_provider`] function.
//! See the example in [`glass_dialog_provider`] for usage details.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, winit};
use tessera_ui_macros::tessera;

use crate::{
    animation,
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    shape_def::Shape,
};

/// The duration of the full dialog animation.
const ANIM_TIME: Duration = Duration::from_millis(300);

/// Arguments for the [`glass_dialog_provider`] component.
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct GlassDialogProviderArgs {
    /// Callback function triggered when a close request is made, for example by
    /// clicking the scrim or pressing the `ESC` key.
    pub on_close_request: Arc<dyn Fn() + Send + Sync>,
}

#[derive(Default)]
pub struct GlassDialogProviderState {
    is_open: bool,
    timer: Option<Instant>,
}

impl GlassDialogProviderState {
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
        if !self.is_open {
            // Already closed, no action needed
        } else {
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
/// * `args` - The arguments for configuring the dialog provider. See [`GlassDialogProviderArgs`].
/// * `main_content` - A closure that renders the main content of the application,
///   which is visible whether the dialog is open or closed.
/// * `dialog_content` - A closure that renders the content of the dialog, which is
///   only visible when `args.is_open` is `true`.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
///
/// use parking_lot::RwLock;
/// use tessera_ui::Color;
/// use tessera_ui_basic_components::{
///     glass_dialog::{GlassDialogProviderArgsBuilder, GlassDialogProviderState, glass_dialog_provider},
///     button::{ButtonArgsBuilder, button},
///     text::{TextArgsBuilder, text},
///     ripple_state::RippleState,
/// };
///
/// #[derive(Default)]
/// struct State {
///     show_dialog: bool,
/// }
///
/// # let state = Arc::new(RwLock::new(State::default()));
/// # let ripple_state = Arc::new(RippleState::default());
/// # let dialog_state = Arc::new(RwLock::new(GlassDialogProviderState::default()));
/// // ...
///
/// glass_dialog_provider(
///     GlassDialogProviderArgsBuilder::default()
///         .on_close_request(Arc::new({
///             let state = state.clone();
///             move || state.write().show_dialog = false
///         }))
///         .build()
///         .unwrap(),
///     dialog_state.clone(),
///     // Main content
///     {
///         let state = state.clone();
///         let ripple = ripple_state.clone();
///         let dialog_state = dialog_state.clone();
///         move || {
///             button(
///                 ButtonArgsBuilder::default()
///                     .on_click(Arc::new(move || {
///                         state.write().show_dialog = true;
///                         dialog_state.write().open();
///                     }))
///                     .build()
///                     .unwrap(),
///                 ripple, // ripple state
///                 || {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("Show Dialog".to_string())
///                             .build()
///                             .unwrap(),
///                     );
///                 },
///             );
///         }
///     },
///     // Dialog content
///     {
///         let state = state.clone();
///         let ripple = ripple_state.clone();
///         let dialog_state = dialog_state.clone();
///         move |content_alpha| {
///             button(
///                 ButtonArgsBuilder::default()
///                     .color(Color::GREEN.with_alpha(content_alpha))
///                     .on_click(Arc::new(move || {
///                         state.write().show_dialog = false;
///                         dialog_state.write().close();
///                     }))
///                     .build()
///                     .unwrap(),
///                 ripple,
///                 || {
///                     text(
///                         TextArgsBuilder::default()
///                             .color(Color::BLACK.with_alpha(content_alpha))
///                             .text("Dialog Content".to_string())
///                             .build()
///                             .unwrap(),
///                     );
///                 },
///             );
///         }
///     },
/// );
/// ```
#[tessera]
pub fn glass_dialog_provider(
    args: GlassDialogProviderArgs,
    state: Arc<RwLock<GlassDialogProviderState>>,
    main_content: impl FnOnce(),
    dialog_content: impl FnOnce(f32),
) {
    // 1. Render the main application content unconditionally.
    main_content();

    // 2. If the dialog is open, render the modal overlay.
    if state.read().is_open
        || state
            .read()
            .timer
            .is_some_and(|timer| timer.elapsed() < ANIM_TIME)
    {
        let on_close_for_keyboard = args.on_close_request.clone();

        let progress = animation::easing(state.read().timer.as_ref().map_or(1.0, |timer| {
            let elapsed = timer.elapsed();
            if elapsed >= ANIM_TIME {
                1.0 // Animation is complete
            } else {
                elapsed.as_secs_f32() / ANIM_TIME.as_secs_f32()
            }
        }));
        let blur_radius = if state.read().is_open {
            progress * 10.0 // Transition from 0 to 10.0 radius
        } else {
            10.0 * (1.0 - progress) // Transition from 10.0 to 0 alpha
        };

        let content_alpha = if state.read().is_open {
            progress * 1.0 // Transition from 0 to 1 alpha
        } else {
            1.0 * (1.0 - progress) // Transition from 1 to 0 alpha
        };

        // 2a. Scrim
        // This Surface covers the entire screen, consuming all mouse clicks
        // and triggering the close request.
        fluid_glass(
            FluidGlassArgsBuilder::default()
                .on_click(args.on_close_request)
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
                .shape(Shape::RoundedRectangle {
                    corner_radius: 0.0,
                    g2_k_value: 3.0,
                })
                .noise_amount(0.0)
                .build()
                .unwrap(),
            None,
            || {},
        );

        // 2b. State Handler for intercepting keyboard events.
        state_handler(Box::new(move |input| {
            // Atomically consume all keyboard events to prevent them from propagating
            // to the main content underneath.
            let events = input.keyboard_events.drain(..).collect::<Vec<_>>();

            // Check the consumed events for the 'Escape' key press.
            for event in events {
                if event.state == winit::event::ElementState::Pressed {
                    if let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) =
                        event.physical_key
                    {
                        (on_close_for_keyboard)();
                    }
                }
            }
        }));

        // 2c. Dialog Content
        // The user-defined dialog content is rendered on top of everything.
        dialog_content(content_alpha);
    }
}
