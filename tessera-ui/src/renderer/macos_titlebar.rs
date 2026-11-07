use objc2::{MainThreadMarker, rc::Retained, sel};
use objc2_app_kit::{NSApplication, NSEvent, NSEventType, NSWindowStyleMask};
use objc2_foundation::{NSUserDefaults, ns_string};

use crate::time::Instant;

#[derive(Default)]
pub(super) struct TitleBarInput {
    presses: Vec<(Instant, Retained<NSEvent>)>,
}

impl TitleBarInput {
    pub(super) fn record_press(&mut self, timestamp: Instant) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        // Redraw can run after AppKit advances to another event.
        if let Some(event) = NSApplication::sharedApplication(mtm)
            .currentEvent()
            .filter(|event| event.r#type() == NSEventType::LeftMouseDown)
        {
            self.presses.push((timestamp, event));
        }
    }

    pub(super) fn handle_press(&self, timestamp: Instant) {
        let Some((_, event)) = self.presses.iter().find(|(press, _)| *press == timestamp) else {
            return;
        };
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let Some(window) = event.window(mtm) else {
            return;
        };
        let style = window.styleMask();
        if style.contains(NSWindowStyleMask::FullScreen) {
            return;
        }
        if event.clickCount() != 2 {
            window.performWindowDragWithEvent(event);
            return;
        }

        let defaults = NSUserDefaults::standardUserDefaults();
        let action = defaults
            .stringForKey(ns_string!("AppleActionOnDoubleClick"))
            .map(|action| action.to_string());
        match action.as_deref() {
            Some("None") => {}
            Some("Minimize") if style.contains(NSWindowStyleMask::Miniaturizable) => {
                window.performMiniaturize(None);
            }
            Some("Fill") if style.contains(NSWindowStyleMask::Resizable) => {
                let app = NSApplication::sharedApplication(mtm);
                // SAFETY: This is AppKit's Window > Move & Resize > Fill action,
                // which takes a single optional sender and targets an NSWindow.
                // Dispatch reports false if this AppKit version cannot handle it.
                if !unsafe { app.sendAction_to_from(sel!(_zoomFill:), Some(&window), None) } {
                    window.performZoom(None);
                }
            }
            Some("Maximize" | "Zoom") | None if style.contains(NSWindowStyleMask::Resizable) => {
                window.performZoom(None);
            }
            _ => {}
        }
    }

    pub(super) fn clear(&mut self) {
        self.presses.clear();
    }
}
