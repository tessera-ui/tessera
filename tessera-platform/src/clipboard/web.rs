//! Browser clipboard events and asynchronous writes.
//!
//! ## Usage
//!
//! Connect focused canvas editors to browser copy, cut, and paste actions.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

use tessera_ui::{
    clipboard::{ClipboardAction, ClipboardResponse, request_clipboard_action},
    winit::{platform::web::WindowExtWebSys, window::Window},
};
use tracing::warn;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    ClipboardEvent, Event, EventTarget, HtmlCanvasElement, KeyboardEvent,
    wasm_bindgen::{JsCast, closure::Closure},
};

struct Listener {
    target: EventTarget,
    name: &'static str,
    callback: Closure<dyn FnMut(Event)>,
    capture: bool,
}

impl Listener {
    fn new(
        target: &EventTarget,
        name: &'static str,
        capture: bool,
        callback: impl FnMut(Event) + 'static,
    ) -> Result<Self, web_sys::wasm_bindgen::JsValue> {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(Event)>);
        target.add_event_listener_with_callback_and_bool(
            name,
            callback.as_ref().unchecked_ref(),
            capture,
        )?;
        Ok(Self {
            target: target.clone(),
            name,
            callback,
            capture,
        })
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        let _ = self.target.remove_event_listener_with_callback_and_bool(
            self.name,
            self.callback.as_ref().unchecked_ref(),
            self.capture,
        );
    }
}

struct BrowserClipboard {
    _listeners: Vec<Listener>,
    window: Arc<Window>,
    restore: Rc<Cell<Option<bool>>>,
}

impl Drop for BrowserClipboard {
    fn drop(&mut self) {
        if let Some(previous) = self.restore.take() {
            self.window.set_prevent_default(previous);
        }
    }
}

thread_local! {
    static BRIDGE: RefCell<Option<BrowserClipboard>> = const { RefCell::new(None) };
}

fn targets_canvas(event: &Event, canvas: &HtmlCanvasElement) -> bool {
    event
        .target()
        .is_some_and(|target| target == canvas.clone().unchecked_into::<EventTarget>())
}

fn is_clipboard_key(event: &KeyboardEvent) -> bool {
    if event.alt_key() {
        return false;
    }
    let command = event.ctrl_key() || event.meta_key();
    (command && matches!(event.key().to_lowercase().as_str(), "c" | "x" | "v"))
        || (event.key() == "Insert" && (event.ctrl_key() || event.shift_key()))
}

pub(super) fn install(window: Arc<Window>) -> Result<(), web_sys::wasm_bindgen::JsValue> {
    clear();
    let Some(canvas) = window.canvas() else {
        return Ok(());
    };
    let Some(document) = canvas.owner_document() else {
        return Ok(());
    };
    let target: EventTarget = document.unchecked_into();
    let mut listeners = Vec::new();
    let restore = Rc::new(Cell::new(None));
    listeners.push(Listener::new(&target, "keydown", true, {
        let canvas = canvas.clone();
        let window = window.clone();
        let restore = restore.clone();
        move |event| {
            if let Some(previous) = restore.take() {
                window.set_prevent_default(previous);
            }
            let Some(key) = event.dyn_ref::<KeyboardEvent>() else {
                return;
            };
            if !targets_canvas(&event, &canvas) || !is_clipboard_key(key) {
                return;
            }
            // Winit normally cancels canvas key defaults. Clipboard shortcuts
            // need their browser action to produce the corresponding event.
            restore.set(Some(window.prevent_default()));
            window.set_prevent_default(false);
        }
    })?);
    listeners.push(Listener::new(&target, "keydown", false, {
        let window = window.clone();
        let restore = restore.clone();
        move |_| {
            if let Some(previous) = restore.take() {
                window.set_prevent_default(previous);
            }
        }
    })?);
    listeners.push(Listener::new(&target, "keyup", true, {
        let window = window.clone();
        let restore = restore.clone();
        move |_| {
            if let Some(previous) = restore.take() {
                window.set_prevent_default(previous);
            }
        }
    })?);
    for name in ["copy", "cut", "paste"] {
        let canvas = canvas.clone();
        listeners.push(Listener::new(&target, name, true, move |event| {
            if !targets_canvas(&event, &canvas) {
                return;
            }
            let Some(event) = event.dyn_ref::<ClipboardEvent>() else {
                return;
            };
            let action = match name {
                "copy" => ClipboardAction::Copy,
                "cut" => ClipboardAction::Cut,
                _ => {
                    let Some(data) = event.clipboard_data() else {
                        return;
                    };
                    let Ok(text) = data.get_data("text/plain") else {
                        return;
                    };
                    if text.is_empty() {
                        return;
                    }
                    ClipboardAction::Paste(text)
                }
            };
            event.prevent_default();
            request_clipboard_action(action, handle_response);
        })?);
    }
    BRIDGE.with_borrow_mut(|bridge| {
        *bridge = Some(BrowserClipboard {
            _listeners: listeners,
            window,
            restore,
        })
    });
    Ok(())
}

pub(super) fn clear() {
    let bridge = BRIDGE.with_borrow_mut(Option::take);
    drop(bridge);
}

fn handle_response(response: ClipboardResponse) {
    if let ClipboardResponse::Write(write) = response {
        write_text(write.text().to_owned(), move |succeeded| {
            write.complete(succeeded)
        });
    }
}

pub(super) fn write_text(text: String, completion: impl FnOnce(bool) + 'static) {
    spawn_local(async move {
        let Some(window) = web_sys::window() else {
            completion(false);
            return;
        };
        let clipboard = window.navigator().clipboard();
        let value: &web_sys::wasm_bindgen::JsValue = clipboard.as_ref();
        if value.is_undefined() || value.is_null() {
            completion(false);
            return;
        }
        let succeeded = JsFuture::from(clipboard.write_text(&text)).await.is_ok();
        if !succeeded {
            warn!("Browser clipboard write was not accepted");
        }
        completion(succeeded);
    });
}
