//! Provides a modal dialog component for overlaying content and intercepting user input.
//!
//! This module defines a dialog provider for creating modal dialogs in UI applications.
//! It allows rendering custom dialog content above the main application, blocking interaction
//! with underlying elements and intercepting keyboard/mouse events (such as ESC or scrim click)
//! to trigger close actions. Typical use cases include confirmation dialogs, alerts, and
//! any scenario requiring user attention before proceeding.
//!
//! The dialog is managed via [`DialogProviderArgs`] and the [`dialog_provider`] function.
//! See the example in [`dialog_provider`] for usage details.

use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{Color, DimensionValue, winit};
use tessera_ui_macros::tessera;

use crate::surface::{SurfaceArgsBuilder, surface};

/// Arguments for the [`dialog_provider`] component.
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct DialogProviderArgs {
    /// Determines whether the dialog is currently visible.
    pub is_open: bool,
    /// Callback function triggered when a close request is made, for example by
    /// clicking the scrim or pressing the `ESC` key.
    pub on_close_request: Arc<dyn Fn() + Send + Sync>,
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
/// * `args` - The arguments for configuring the dialog provider. See [`DialogProviderArgs`].
/// * `main_content` - A closure that renders the main content of the application,
///   which is visible whether the dialog is open or closed.
/// * `dialog_content` - A closure that renders the content of the dialog, which is
///   only visible when `args.is_open` is `true`.
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::{Arc, RwLock};
///
/// use tessera_ui::use_state;
/// use tessera_ui_basic_components::{
///     button::button,
///     dialog::{dialog_provider, Dialog, DialogArgs},
///     text::text,
/// };
///
/// let is_open = use_state(|| false);
/// let dialog_state = use_state(Vec::new);
///
/// dialog_provider(dialog_state.clone());
///
/// button("Open Dialog", || {
///     let mut dialogs = dialog_state.write();
///     dialogs.push(Dialog {
///         modal: true,
///         ui: Arc::new(move || {
///             text("This is a dialog".to_string());
///             button("close", || {});
///         }),
///     });
/// });
/// ```
#[tessera]
pub fn dialog_provider(
    args: DialogProviderArgs,
    main_content: impl FnOnce(),
    dialog_content: impl FnOnce(),
) {
    // 1. Render the main application content unconditionally.
    main_content();

    // 2. If the dialog is open, render the modal overlay.
    if args.is_open {
        let on_close_for_keyboard = args.on_close_request.clone();

        // 2a. Scrim
        // This Surface covers the entire screen, consuming all mouse clicks
        // and triggering the close request.
        surface(
            SurfaceArgsBuilder::default()
                .color(Color::BLACK.with_alpha(0.5))
                .on_click(Some(args.on_close_request))
                .width(DimensionValue::Fill {
                    min: None,
                    max: None,
                })
                .height(DimensionValue::Fill {
                    min: None,
                    max: None,
                })
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
        dialog_content();
    }
}
