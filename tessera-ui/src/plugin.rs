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
    fn on_resumed(&self, _context: &PluginContext) -> PluginResult {
        Ok(())
    }

    /// Called when the renderer suspends and releases platform resources.
    fn on_suspended(&self, _context: &PluginContext) -> PluginResult {
        Ok(())
    }

    /// Called when the renderer is shutting down.
    fn on_shutdown(&self, _context: &PluginContext) -> PluginResult {
        Ok(())
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
    register_plugin_arc(Arc::new(plugin));
}

/// Registers a plugin instance wrapped in an `Arc`.
pub fn register_plugin_boxed<P: Plugin>(plugin: Arc<P>) {
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
    let registry = plugin_instance_registry().read();
    let type_id = TypeId::of::<T>();
    let Some(plugin) = registry.get(&type_id) else {
        panic!("Plugin '{}' is not registered", std::any::type_name::<T>());
    };
    let Some(plugin) = plugin.as_ref().downcast_ref::<T>() else {
        panic!(
            "Plugin '{}' has a mismatched type",
            std::any::type_name::<T>()
        );
    };
    f(plugin)
}

pub(crate) struct PluginHost {
    plugins: Vec<Arc<dyn Plugin>>,
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
        self.dispatch("resumed", context, Plugin::on_resumed);
    }

    pub(crate) fn suspended(&self, context: &PluginContext) {
        self.dispatch("suspended", context, Plugin::on_suspended);
    }

    pub(crate) fn shutdown(&mut self, context: &PluginContext) {
        if self.shutdown_called {
            return;
        }
        self.shutdown_called = true;
        self.dispatch("shutdown", context, Plugin::on_shutdown);
    }

    fn dispatch(
        &self,
        stage: &'static str,
        context: &PluginContext,
        handler: fn(&dyn Plugin, &PluginContext) -> PluginResult,
    ) {
        for plugin in &self.plugins {
            if let Err(err) = handler(plugin.as_ref(), context) {
                error!("Plugin '{}' {} hook failed: {}", plugin.name(), stage, err);
            }
        }
    }
}

fn plugin_registry() -> &'static RwLock<Vec<Arc<dyn Plugin>>> {
    static REGISTRY: OnceLock<RwLock<Vec<Arc<dyn Plugin>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(Vec::new()))
}

fn registered_plugins() -> Vec<Arc<dyn Plugin>> {
    plugin_registry().read().clone()
}

fn register_plugin_arc<P: Plugin>(plugin: Arc<P>) {
    let mut registry = plugin_registry().write();
    registry.push(plugin.clone() as Arc<dyn Plugin>);

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
