//! Desktop window services for Tessera platform plugins.
//!
//! ## Usage
//!
//! Control desktop window state from app actions and custom title bars.

use tessera_ui::{DesktopPlatformContext, PluginContext};

/// Runs a closure with desktop platform access when available.
pub fn with_desktop<R>(f: impl FnOnce(&DesktopPlatformContext) -> R) -> Option<R> {
    PluginContext::current().map(|context| f(context.desktop()))
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
