use std::sync::Arc;

use derive_builder::Builder;
use tessera::{Color, DimensionValue, winit};
use tessera_macros::tessera;

use crate::surface::{SurfaceArgsBuilder, surface};

/// Arguments for the `dialog_provider` component.
#[derive(Builder)]
pub struct DialogProviderArgs {
    /// Determines whether the dialog is currently visible.
    pub is_open: bool,
    /// Callback function triggered when a close request is made (e.g., by clicking the scrim or pressing ESC).
    pub on_close_request: Arc<dyn Fn() + Send + Sync>,
}

/// A provider component that manages the rendering and event flow for a modal dialog.
///
/// This component should be used as one of the outermost layers of the application.
/// It renders the main content, and when `is_open` is true, it overlays a modal
/// dialog, intercepting all input.
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
                .color(Color::BLACK)
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
