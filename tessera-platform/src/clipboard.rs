//! Clipboard access for Tessera platform plugins.
//!
//! ## Usage
//!
//! Enable copy and paste in text inputs and editors.
use std::sync::{Arc, OnceLock};

use parking_lot::RwLock;
use tessera_ui::{Plugin, PluginContext, PluginResult};
use tracing::warn;

#[cfg(target_os = "android")]
use tessera_ui::android::{ActivityRef, activity};
#[cfg(target_os = "android")]
use tessera_ui::winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
tessera_ui::android::jni_bind! {
    class "com.tessera.platform.ClipboardPlugin" as ClipboardPluginJni {
        /// Whether the clipboard contains text.
        fn hasText(activity: ActivityRef) -> bool;
        /// Returns clipboard text.
        fn getText(activity: ActivityRef) -> String;
        /// Sets clipboard text.
        fn setText(activity: ActivityRef, text: &str) -> ();
        /// Clears clipboard contents.
        fn clear(activity: ActivityRef) -> ();
    }
}

/// Clipboard plugin that wires platform clipboard services.
#[derive(Clone, Debug)]
pub struct ClipboardPlugin;

impl ClipboardPlugin {
    /// Creates a clipboard plugin.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClipboardPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ClipboardPlugin {
    fn on_resumed(&mut self, context: &PluginContext) -> PluginResult {
        let mut state = clipboard_state().write();
        state.clipboard = Clipboard::new(context);
        Ok(())
    }

    fn on_suspended(&mut self, _context: &PluginContext) -> PluginResult {
        clipboard_state().write().clipboard = None;
        Ok(())
    }

    fn on_shutdown(&mut self, _context: &PluginContext) -> PluginResult {
        clipboard_state().write().clipboard = None;
        Ok(())
    }
}

/// Clipboard handle backed by platform-specific implementations.
pub struct Clipboard {
    #[cfg(not(target_os = "android"))]
    manager: arboard::Clipboard,
    #[cfg(target_os = "android")]
    android_app: AndroidApp,
}

impl Clipboard {
    #[cfg(not(target_os = "android"))]
    fn new(_context: &PluginContext) -> Option<Self> {
        match arboard::Clipboard::new() {
            Ok(manager) => Some(Self { manager }),
            Err(err) => {
                warn!("Failed to initialize clipboard: {err}");
                None
            }
        }
    }

    #[cfg(target_os = "android")]
    fn new(context: &PluginContext) -> Option<Self> {
        Some(Self {
            android_app: context.android_app().clone(),
        })
    }

    /// Sets clipboard text, replacing previous content.
    pub fn set_text(&mut self, text: &str) {
        #[cfg(not(target_os = "android"))]
        {
            let _ = self.manager.set_text(text.to_string());
        }
        #[cfg(target_os = "android")]
        {
            let activity = activity(&self.android_app);
            if let Err(err) = ClipboardPluginJni::setText(&self.android_app, activity, text) {
                warn!("Android clipboard set_text failed: {err}");
            }
        }
    }

    /// Returns clipboard text when available.
    pub fn get_text(&mut self) -> Option<String> {
        #[cfg(not(target_os = "android"))]
        {
            self.manager.get_text().ok()
        }
        #[cfg(target_os = "android")]
        {
            let activity = activity(&self.android_app);
            let has_text = match ClipboardPluginJni::hasText(&self.android_app, activity) {
                Ok(value) => value,
                Err(err) => {
                    warn!("Android clipboard has_text failed: {err}");
                    return None;
                }
            };
            if !has_text {
                return None;
            }
            match ClipboardPluginJni::getText(&self.android_app, activity) {
                Ok(text) => Some(text),
                Err(err) => {
                    warn!("Android clipboard get_text failed: {err}");
                    None
                }
            }
        }
    }

    /// Clears clipboard contents.
    pub fn clear(&mut self) {
        #[cfg(not(target_os = "android"))]
        {
            let _ = self.manager.clear();
        }
        #[cfg(target_os = "android")]
        {
            let activity = activity(&self.android_app);
            if let Err(err) = ClipboardPluginJni::clear(&self.android_app, activity) {
                warn!("Android clipboard clear failed: {err}");
            }
        }
    }
}

#[derive(Default)]
struct ClipboardState {
    clipboard: Option<Clipboard>,
}

fn clipboard_state() -> &'static Arc<RwLock<ClipboardState>> {
    static STATE: OnceLock<Arc<RwLock<ClipboardState>>> = OnceLock::new();
    STATE.get_or_init(|| Arc::new(RwLock::new(ClipboardState::default())))
}

/// Runs a closure with mutable clipboard access when available.
pub fn with_clipboard_mut<R>(f: impl FnOnce(&mut Clipboard) -> R) -> Option<R> {
    let mut state = clipboard_state().write();
    let clipboard = state.clipboard.as_mut()?;
    Some(f(clipboard))
}

/// Sets the clipboard text when clipboard access is available.
pub fn set_text(text: &str) {
    let _ = with_clipboard_mut(|clipboard| clipboard.set_text(text));
}

/// Returns clipboard text when clipboard access is available.
pub fn get_text() -> Option<String> {
    with_clipboard_mut(|clipboard| clipboard.get_text()).flatten()
}

/// Clears clipboard contents when clipboard access is available.
pub fn clear() {
    let _ = with_clipboard_mut(|clipboard| clipboard.clear());
}
