//! Plugin lifecycle hooks for Tessera platform integrations.

use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    error::Error,
    rc::Rc,
    sync::Arc,
};

use tracing::{error, warn};
use winit::window::Window;

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

/// The result type used by plugin lifecycle hooks.
pub type PluginResult = Result<(), Box<dyn Error + Send + Sync>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DesktopWindowAction {
    Minimize,
    Maximize,
    ToggleMaximize,
    Close,
}

impl DesktopWindowAction {
    pub(crate) fn merge_pending(current: Option<Self>, new: Self) -> Self {
        match (current, new) {
            (Some(Self::Close), _) | (_, Self::Close) => Self::Close,
            (_, new) => new,
        }
    }
}

/// Desktop platform services exposed to plugins.
#[derive(Clone)]
pub struct DesktopPlatformContext {
    window: Arc<Window>,
    pending_action: Rc<Cell<Option<DesktopWindowAction>>>,
    wake: Rc<dyn Fn()>,
}

impl DesktopPlatformContext {
    /// Returns the active window associated with the renderer.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Clones the underlying window handle for long-lived usage.
    pub fn window_handle(&self) -> Arc<Window> {
        self.window.clone()
    }

    /// Minimizes the current window.
    pub fn minimize(&self) {
        self.request_action(DesktopWindowAction::Minimize);
    }

    /// Maximizes the current window.
    pub fn maximize(&self) {
        self.request_action(DesktopWindowAction::Maximize);
    }

    /// Toggles the maximized state of the current window.
    pub fn toggle_maximize(&self) {
        self.request_action(DesktopWindowAction::ToggleMaximize);
    }

    /// Requests host-managed application shutdown.
    pub fn request_close(&self) {
        self.request_action(DesktopWindowAction::Close);
    }

    fn request_action(&self, action: DesktopWindowAction) {
        let current = self.pending_action.get();
        let next = DesktopWindowAction::merge_pending(current, action);
        self.pending_action.set(Some(next));
        if current != Some(next) {
            (self.wake)();
        }
    }

    pub(crate) fn new(
        window: Arc<Window>,
        pending_action: Rc<Cell<Option<DesktopWindowAction>>>,
        wake: Rc<dyn Fn()>,
    ) -> Self {
        Self {
            window,
            pending_action,
            wake,
        }
    }
}

/// Lifecycle hooks for platform plugins.
///
/// Plugins live in their UI session; hooks and [`with_plugin`] run on the UI
/// thread.
pub trait Plugin: 'static {
    /// Called when the renderer creates or resumes its platform resources.
    fn on_resumed(&mut self, _context: &PluginContext) -> PluginResult {
        Ok(())
    }

    /// Called when the renderer suspends and releases platform resources.
    fn on_suspended(&mut self, _context: &PluginContext) -> PluginResult {
        Ok(())
    }

    /// Called when the renderer is shutting down.
    fn on_shutdown(&mut self, _context: &PluginContext) -> PluginResult {
        Ok(())
    }
}

trait PluginEntry {
    fn name(&self) -> &'static str;
    fn resumed(&self, context: &PluginContext) -> PluginResult;
    fn suspended(&self, context: &PluginContext) -> PluginResult;
    fn shutdown(&self, context: &PluginContext) -> PluginResult;
    fn as_any(&self) -> &dyn Any;
}

impl<P: Plugin> PluginEntry for RefCell<P> {
    fn name(&self) -> &'static str {
        std::any::type_name::<P>()
    }

    fn resumed(&self, context: &PluginContext) -> PluginResult {
        self.borrow_mut().on_resumed(context)
    }

    fn suspended(&self, context: &PluginContext) -> PluginResult {
        self.borrow_mut().on_suspended(context)
    }

    fn shutdown(&self, context: &PluginContext) -> PluginResult {
        self.borrow_mut().on_shutdown(context)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Platform context shared with plugins while the renderer is resumed.
#[derive(Clone)]
pub struct PluginContext {
    desktop: DesktopPlatformContext,
    #[cfg(target_os = "android")]
    android_app: AndroidApp,
}

impl PluginContext {
    /// Returns the context of the active session while its renderer is
    /// resumed.
    pub fn current() -> Option<Self> {
        crate::RuntimeSession::current()?;
        crate::local_runtime_resource::<Plugins>()
            .context
            .borrow()
            .clone()
    }

    /// Returns desktop platform services associated with the renderer.
    pub fn desktop(&self) -> &DesktopPlatformContext {
        &self.desktop
    }

    /// Returns the active window associated with the renderer.
    pub fn window(&self) -> &Window {
        self.desktop.window()
    }

    /// Clones the underlying window handle for long-lived usage.
    pub fn window_handle(&self) -> Arc<Window> {
        self.desktop.window_handle()
    }

    /// Returns the active UIKit view when called on the main thread.
    #[cfg(target_os = "ios")]
    pub fn ui_view(&self) -> Option<objc2::rc::Retained<objc2_ui_kit::UIView>> {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

        objc2::MainThreadMarker::new()?;
        let RawWindowHandle::UiKit(handle) = self.window().window_handle().ok()?.as_raw() else {
            return None;
        };
        // SAFETY: Winit owns this live view and the main-thread check precedes access.
        unsafe { objc2::rc::Retained::retain(handle.ui_view.as_ptr().cast()) }
    }

    /// Returns the Android application handle when running on Android.
    #[cfg(target_os = "android")]
    pub fn android_app(&self) -> &AndroidApp {
        &self.android_app
    }

    #[cfg(target_os = "android")]
    pub(crate) fn new(desktop: DesktopPlatformContext, android_app: AndroidApp) -> Self {
        Self {
            desktop,
            android_app,
        }
    }

    #[cfg(not(target_os = "android"))]
    pub(crate) fn new(desktop: DesktopPlatformContext) -> Self {
        Self { desktop }
    }
}

#[derive(Default)]
struct Plugins {
    entries: RefCell<Vec<(TypeId, Rc<dyn PluginEntry>)>>,
    context: RefCell<Option<PluginContext>>,
    shut_down: Cell<bool>,
}

impl Plugins {
    fn entries(&self) -> Vec<Rc<dyn PluginEntry>> {
        self.entries
            .borrow()
            .iter()
            .map(|(_, entry)| entry.clone())
            .collect()
    }

    fn dispatch(
        &self,
        stage: &'static str,
        context: &PluginContext,
        hook: impl Fn(&dyn PluginEntry, &PluginContext) -> PluginResult,
    ) {
        for plugin in self.entries() {
            if let Err(err) = hook(plugin.as_ref(), context) {
                error!("Plugin '{}' {} hook failed: {}", plugin.name(), stage, err);
            }
        }
    }
}

pub(crate) fn register_plugin<P: Plugin>(plugin: P) {
    assert!(
        crate::RuntimeSession::current().is_some(),
        "register plugins through an EntryPoint"
    );
    let plugins = crate::local_runtime_resource::<Plugins>();
    let mut entries = plugins.entries.borrow_mut();
    let type_id = TypeId::of::<P>();
    if entries.iter().any(|(registered, _)| *registered == type_id) {
        warn!(
            "Plugin '{}' was registered more than once; keeping the first instance",
            std::any::type_name::<P>()
        );
        return;
    }
    entries.push((type_id, Rc::new(RefCell::new(plugin))));
}

pub(crate) fn plugins_resumed(context: PluginContext) {
    let plugins = crate::local_runtime_resource::<Plugins>();
    plugins.context.replace(Some(context.clone()));
    plugins.dispatch("resumed", &context, |plugin, context| {
        plugin.resumed(context)
    });
}

pub(crate) fn plugins_suspended(context: &PluginContext) {
    let plugins = crate::local_runtime_resource::<Plugins>();
    plugins.dispatch("suspended", context, |plugin, context| {
        plugin.suspended(context)
    });
    plugins.context.replace(None);
}

pub(crate) fn plugins_shutdown(context: &PluginContext) {
    let plugins = crate::local_runtime_resource::<Plugins>();
    if plugins.shut_down.replace(true) {
        return;
    }
    plugins.dispatch("shutdown", context, |plugin, context| {
        plugin.shutdown(context)
    });
    plugins.context.replace(None);
}

/// Runs `access` with the registered plugin of type `P` in the active session.
///
/// Returns `None` when no such plugin is registered.
///
/// # Panics
///
/// Panics when called again for the same plugin from inside `access` or one of
/// its hooks.
pub fn with_plugin<P: Plugin, R>(access: impl FnOnce(&mut P) -> R) -> Option<R> {
    crate::RuntimeSession::current()?;
    let plugin = crate::local_runtime_resource::<Plugins>()
        .entries
        .borrow()
        .iter()
        .find(|(type_id, _)| *type_id == TypeId::of::<P>())
        .map(|(_, entry)| entry.clone())?;
    let plugin = plugin
        .as_any()
        .downcast_ref::<RefCell<P>>()
        .expect("plugin entries are keyed by their type");
    let mut plugin = plugin
        .try_borrow_mut()
        .unwrap_or_else(|_| panic!("plugin '{}' is already in use", std::any::type_name::<P>()));
    Some(access(&mut plugin))
}
