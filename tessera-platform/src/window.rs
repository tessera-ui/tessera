//! Desktop window services for Tessera platform plugins.
//!
//! ## Usage
//!
//! Control desktop window state from app actions and custom title bars.

use std::sync::{Arc, OnceLock};

use parking_lot::RwLock;
use tessera_ui::{DesktopPlatformContext, Plugin, PluginContext, PluginResult};

/// Window plugin that wires desktop platform window services.
#[derive(Clone, Debug)]
pub struct WindowPlugin;

impl WindowPlugin {
    /// Creates a window plugin.
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for WindowPlugin {
    fn on_resumed(&mut self, context: &PluginContext) -> PluginResult {
        window_state().write().desktop = Some(context.desktop().clone());
        Ok(())
    }

    fn on_suspended(&mut self, _context: &PluginContext) -> PluginResult {
        window_state().write().desktop = None;
        Ok(())
    }

    fn on_shutdown(&mut self, _context: &PluginContext) -> PluginResult {
        window_state().write().desktop = None;
        Ok(())
    }
}

#[derive(Default)]
struct WindowState {
    desktop: Option<DesktopPlatformContext>,
}

fn window_state() -> &'static Arc<RwLock<WindowState>> {
    static STATE: OnceLock<Arc<RwLock<WindowState>>> = OnceLock::new();
    STATE.get_or_init(|| Arc::new(RwLock::new(WindowState::default())))
}

/// Runs a closure with desktop platform access when available.
pub fn with_desktop<R>(f: impl FnOnce(&DesktopPlatformContext) -> R) -> Option<R> {
    let state = window_state().read();
    let desktop = state.desktop.as_ref()?;
    Some(f(desktop))
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
