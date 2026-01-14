//! Plugin lifecycle hooks for Tessera platform integrations.
//!
//! ## Usage
//!
//! Register plugins and services for platform-specific capabilities.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    error::Error,
    sync::{Arc, OnceLock},
};

use parking_lot::RwLock;
use tracing::error;
use winit::window::Window;

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

/// The result type used by plugin lifecycle hooks.
pub type PluginResult = Result<(), Box<dyn Error + Send + Sync>>;

/// Lifecycle hooks for platform plugins.
pub trait Plugin: Send + Sync + 'static {
    /// Returns the plugin name for logging and diagnostics.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

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

trait PluginEntry: Send + Sync {
    fn name(&self) -> &'static str;
    fn resumed(&self, context: &PluginContext) -> PluginResult;
    fn suspended(&self, context: &PluginContext) -> PluginResult;
    fn shutdown(&self, context: &PluginContext) -> PluginResult;
}

struct PluginSlot<P: Plugin> {
    inner: Arc<RwLock<P>>,
}

impl<P: Plugin> PluginSlot<P> {
    fn new(inner: Arc<RwLock<P>>) -> Self {
        Self { inner }
    }
}

impl<P: Plugin> PluginEntry for PluginSlot<P> {
    fn name(&self) -> &'static str {
        self.inner.read().name()
    }

    fn resumed(&self, context: &PluginContext) -> PluginResult {
        self.inner.write().on_resumed(context)
    }

    fn suspended(&self, context: &PluginContext) -> PluginResult {
        self.inner.write().on_suspended(context)
    }

    fn shutdown(&self, context: &PluginContext) -> PluginResult {
        self.inner.write().on_shutdown(context)
    }
}

/// Platform context shared with plugins during lifecycle events.
#[derive(Clone)]
pub struct PluginContext {
    window: Arc<Window>,
    #[cfg(target_os = "android")]
    android_app: AndroidApp,
}

impl PluginContext {
    /// Returns the active window associated with the renderer.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Clones the underlying window handle for long-lived usage.
    pub fn window_handle(&self) -> Arc<Window> {
        self.window.clone()
    }

    /// Returns the Android application handle when running on Android.
    #[cfg(target_os = "android")]
    pub fn android_app(&self) -> &AndroidApp {
        &self.android_app
    }

    #[cfg(target_os = "android")]
    pub(crate) fn new(window: Arc<Window>, android_app: AndroidApp) -> Self {
        Self {
            window,
            android_app,
        }
    }

    #[cfg(not(target_os = "android"))]
    pub(crate) fn new(window: Arc<Window>) -> Self {
        Self { window }
    }
}

/// Registers a plugin instance for the current process.
pub fn register_plugin<P: Plugin>(plugin: P) {
    register_plugin_arc(Arc::new(RwLock::new(plugin)));
}

/// Registers a plugin instance wrapped in an `Arc<RwLock<_>>`.
pub fn register_plugin_boxed<P: Plugin>(plugin: Arc<RwLock<P>>) {
    register_plugin_arc(plugin);
}

/// Provides access to the registered plugin instance.
///
/// # Panics
///
/// Panics if the plugin type was not registered.
pub fn with_plugin<T, R>(f: impl FnOnce(&T) -> R) -> R
where
    T: Plugin + 'static,
{
    let plugin = plugin_instance::<T>();
    let guard = plugin.read();
    f(&*guard)
}

/// Provides mutable access to the registered plugin instance.
///
/// # Panics
///
/// Panics if the plugin type was not registered.
pub fn with_plugin_mut<T, R>(f: impl FnOnce(&mut T) -> R) -> R
where
    T: Plugin + 'static,
{
    let plugin = plugin_instance::<T>();
    let mut guard = plugin.write();
    f(&mut *guard)
}

pub(crate) struct PluginHost {
    plugins: Vec<Arc<dyn PluginEntry>>,
    shutdown_called: bool,
}

impl PluginHost {
    pub(crate) fn new() -> Self {
        Self {
            plugins: registered_plugins(),
            shutdown_called: false,
        }
    }

    pub(crate) fn resumed(&self, context: &PluginContext) {
        self.dispatch("resumed", context, |plugin, ctx| plugin.resumed(ctx));
    }

    pub(crate) fn suspended(&self, context: &PluginContext) {
        self.dispatch("suspended", context, |plugin, ctx| plugin.suspended(ctx));
    }

    pub(crate) fn shutdown(&mut self, context: &PluginContext) {
        if self.shutdown_called {
            return;
        }
        self.shutdown_called = true;
        self.dispatch("shutdown", context, |plugin, ctx| plugin.shutdown(ctx));
    }

    fn dispatch<F>(&self, stage: &'static str, context: &PluginContext, mut handler: F)
    where
        F: FnMut(&dyn PluginEntry, &PluginContext) -> PluginResult,
    {
        for plugin in &self.plugins {
            if let Err(err) = handler(plugin.as_ref(), context) {
                error!("Plugin '{}' {} hook failed: {}", plugin.name(), stage, err);
            }
        }
    }
}

fn plugin_registry() -> &'static RwLock<Vec<Arc<dyn PluginEntry>>> {
    static REGISTRY: OnceLock<RwLock<Vec<Arc<dyn PluginEntry>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(Vec::new()))
}

fn registered_plugins() -> Vec<Arc<dyn PluginEntry>> {
    plugin_registry().read().clone()
}

fn register_plugin_arc<P: Plugin>(plugin: Arc<RwLock<P>>) {
    let mut registry = plugin_registry().write();
    registry.push(Arc::new(PluginSlot::new(plugin.clone())) as Arc<dyn PluginEntry>);

    let mut instances = plugin_instance_registry().write();
    let type_id = TypeId::of::<P>();
    if instances.contains_key(&type_id) {
        panic!(
            "Plugin '{}' was registered more than once",
            std::any::type_name::<P>()
        );
    }
    instances.insert(type_id, plugin as Arc<dyn Any + Send + Sync>);
}

fn plugin_instance_registry() -> &'static RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>> {
    static REGISTRY: OnceLock<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> =
        OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

fn plugin_instance<T: Plugin>() -> Arc<RwLock<T>> {
    let registry = plugin_instance_registry().read();
    let type_id = TypeId::of::<T>();
    let Some(plugin) = registry.get(&type_id) else {
        panic!("Plugin '{}' is not registered", std::any::type_name::<T>());
    };
    let plugin = plugin.clone();
    drop(registry);
    match Arc::downcast::<RwLock<T>>(plugin) {
        Ok(plugin) => plugin,
        Err(_) => panic!(
            "Plugin '{}' has a mismatched type",
            std::any::type_name::<T>()
        ),
    }
}
