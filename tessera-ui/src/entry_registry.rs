//! Entry package registry for renderer startup.
use crate::{
    plugin::{Plugin, register_plugin},
    render_module::RenderModule,
};

/// Registers modules and plugins for a Tessera entry point.
pub trait TesseraPackage {
    /// Registers this package into the provided registry.
    fn register(self, registry: &mut EntryRegistry);
}

/// Collects modules and registers plugins for an application entry.
#[derive(Default)]
pub struct EntryRegistry {
    modules: Vec<Box<dyn RenderModule>>,
}

impl EntryRegistry {
    /// Creates a new registry for entry-time registration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a render module to the entry registry.
    pub fn add_module(&mut self, module: impl RenderModule + 'static) {
        self.modules.push(Box::new(module));
    }

    /// Registers a plugin instance with the application session.
    pub fn register_plugin<P: Plugin>(&mut self, plugin: P) {
        register_plugin(plugin);
    }

    /// Registers a package into the entry registry.
    pub fn register_package<P: TesseraPackage>(&mut self, package: P) {
        package.register(self);
    }

    /// Finalizes the registry into a module list for the renderer.
    pub fn finish(self) -> Vec<Box<dyn RenderModule>> {
        self.modules
    }
}
