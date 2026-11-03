//! Desktop window services for Tessera platform plugins.
//!
//! ## Usage
//!
//! Control desktop window state from app actions and custom title bars.

use tessera_ui::{DesktopPlatformContext, Plugin, PluginContext, PluginResult, try_with_plugin};

/// Window plugin that wires desktop platform window services.
#[derive(Clone, Default)]
pub struct WindowPlugin {
    desktop: Option<DesktopPlatformContext>,
}

impl std::fmt::Debug for WindowPlugin {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WindowPlugin")
            .field("available", &self.desktop.is_some())
            .finish()
    }
}

impl WindowPlugin {
    /// Creates a window plugin.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Plugin for WindowPlugin {
    fn on_resumed(&mut self, context: &PluginContext) -> PluginResult {
        self.desktop = Some(context.desktop().clone());
        Ok(())
    }

    fn on_suspended(&mut self, _context: &PluginContext) -> PluginResult {
        self.desktop = None;
        Ok(())
    }

    fn on_shutdown(&mut self, _context: &PluginContext) -> PluginResult {
        self.desktop = None;
        Ok(())
    }
}

/// Runs a closure with desktop platform access when available.
pub fn with_desktop<R>(f: impl FnOnce(&DesktopPlatformContext) -> R) -> Option<R> {
    let desktop = try_with_plugin::<WindowPlugin, _>(|plugin| plugin.desktop.clone()).flatten()?;
    Some(f(&desktop))
}

/// Minimizes the current application window when desktop services are
/// available.
pub fn minimize() {
    let _ = with_desktop(|desktop| desktop.minimize());
}

/// Maximizes the current application window when desktop services are
/// available.
pub fn maximize() {
    let _ = with_desktop(|desktop| desktop.maximize());
}

/// Toggles the current application window maximized state when desktop services
/// are available.
pub fn toggle_maximize() {
    let _ = with_desktop(|desktop| desktop.toggle_maximize());
}

/// Requests application shutdown through the host when desktop services are
/// available.
pub fn close() {
    let _ = with_desktop(|desktop| desktop.request_close());
}
