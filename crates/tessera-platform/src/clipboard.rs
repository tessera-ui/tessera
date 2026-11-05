//! Clipboard access for Tessera platform plugins.
//!
//! ## Usage
//!
//! Enable copy and paste in text inputs and editors.
use tessera_ui::{Plugin, PluginContext, PluginResult, try_with_plugin_mut};

#[cfg(target_family = "wasm")]
mod web;

#[cfg(not(any(target_family = "wasm", target_os = "ios")))]
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
#[derive(Default)]
pub struct ClipboardPlugin {
    clipboard: Option<Clipboard>,
}

impl std::fmt::Debug for ClipboardPlugin {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ClipboardPlugin")
            .field("available", &self.clipboard.is_some())
            .finish()
    }
}

impl ClipboardPlugin {
    /// Creates a clipboard plugin.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Plugin for ClipboardPlugin {
    fn on_resumed(&mut self, context: &PluginContext) -> PluginResult {
        #[cfg(target_family = "wasm")]
        web::install(context.window_handle())
            .map_err(|_| "Failed to install browser clipboard events")?;
        self.clipboard = Clipboard::new(context);
        Ok(())
    }

    fn on_suspended(&mut self, _context: &PluginContext) -> PluginResult {
        #[cfg(target_family = "wasm")]
        web::clear();
        self.clipboard = None;
        Ok(())
    }

    fn on_shutdown(&mut self, _context: &PluginContext) -> PluginResult {
        #[cfg(target_family = "wasm")]
        web::clear();
        self.clipboard = None;
        Ok(())
    }
}

/// Clipboard handle backed by platform-specific implementations.
pub struct Clipboard {
    #[cfg(not(any(target_os = "android", target_os = "ios", target_family = "wasm")))]
    manager: arboard::Clipboard,
    #[cfg(target_os = "android")]
    android_app: AndroidApp,
}

impl Clipboard {
    #[cfg(not(any(target_os = "android", target_os = "ios", target_family = "wasm")))]
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

    #[cfg(any(target_family = "wasm", target_os = "ios"))]
    fn new(_context: &PluginContext) -> Option<Self> {
        Some(Self {})
    }

    /// Sets clipboard text, replacing previous content.
    pub fn set_text(&mut self, text: &str) {
        #[cfg(target_os = "ios")]
        if objc2::MainThreadMarker::new().is_some() {
            // SAFETY: Pasteboard properties are accessed serially on the UI thread.
            unsafe {
                objc2_ui_kit::UIPasteboard::generalPasteboard()
                    .setString(Some(&objc2_foundation::NSString::from_str(text)));
            }
        }
        #[cfg(not(any(target_os = "android", target_os = "ios", target_family = "wasm")))]
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
        #[cfg(target_family = "wasm")]
        {
            web::write_text(text.to_owned(), |_| {});
        }
    }

    /// Returns clipboard text when available synchronously.
    ///
    /// Browser paste text is delivered through clipboard input events instead.
    pub fn get_text(&mut self) -> Option<String> {
        #[cfg(target_os = "ios")]
        {
            objc2::MainThreadMarker::new()?;
            // SAFETY: Pasteboard properties are accessed serially on the UI thread.
            return unsafe { objc2_ui_kit::UIPasteboard::generalPasteboard().string() }
                .map(|text| text.to_string());
        }
        #[cfg(not(any(target_os = "android", target_os = "ios", target_family = "wasm")))]
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
        #[cfg(target_family = "wasm")]
        {
            None
        }
    }

    /// Clears clipboard contents.
    pub fn clear(&mut self) {
        #[cfg(target_os = "ios")]
        if objc2::MainThreadMarker::new().is_some() {
            // SAFETY: Pasteboard properties are accessed serially on the UI thread.
            unsafe {
                objc2_ui_kit::UIPasteboard::generalPasteboard()
                    .setItems(&objc2_foundation::NSArray::new());
            }
        }
        #[cfg(not(any(target_os = "android", target_os = "ios", target_family = "wasm")))]
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
        #[cfg(target_family = "wasm")]
        {
            web::write_text(String::new(), |_| {});
        }
    }
}

/// Runs a closure with mutable clipboard access when available.
pub fn with_clipboard_mut<R>(f: impl FnOnce(&mut Clipboard) -> R) -> Option<R> {
    try_with_plugin_mut::<ClipboardPlugin, _>(|plugin| plugin.clipboard.as_mut().map(f)).flatten()
}

/// Sets the clipboard text when clipboard access is available.
pub fn set_text(text: &str) {
    let _ = with_clipboard_mut(|clipboard| clipboard.set_text(text));
}

/// Returns clipboard text when synchronous clipboard access is available.
///
/// Browser paste text is delivered through clipboard input events instead.
pub fn get_text() -> Option<String> {
    with_clipboard_mut(|clipboard| clipboard.get_text()).flatten()
}

/// Clears clipboard contents when clipboard access is available.
pub fn clear() {
    let _ = with_clipboard_mut(|clipboard| clipboard.clear());
}
