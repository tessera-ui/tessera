//! Application entry builder for registering packages and launching the
//! renderer.
//!
//! ## Usage
//!
//! Configure app startup with packages and plugins before launching the
//! renderer.

use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    entry_registry::{EntryRegistry, TesseraPackage},
    plugin::Plugin,
    render_module::RenderModule,
    renderer::{Renderer, TesseraConfig},
};

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

/// Builder for application entry configuration and startup.
pub struct EntryPoint {
    entry: Box<dyn Fn()>,
    registry: EntryRegistry,
    config: TesseraConfig,
}

impl EntryPoint {
    /// Creates a new entry point builder from the root UI function.
    pub fn new(entry: impl Fn() + 'static) -> Self {
        Self {
            entry: Box::new(entry),
            registry: EntryRegistry::new(),
            config: TesseraConfig::default(),
        }
    }

    /// Adds a render module to the entry registry.
    pub fn module(mut self, module: impl RenderModule + 'static) -> Self {
        self.registry.add_module(module);
        self
    }

    /// Registers a plugin instance with the global plugin registry.
    pub fn plugin(mut self, plugin: impl Plugin) -> Self {
        self.registry.register_plugin(plugin);
        self
    }

    /// Registers a boxed plugin instance with the global plugin registry.
    pub fn plugin_boxed<P: Plugin>(mut self, plugin: Arc<RwLock<P>>) -> Self {
        self.registry.register_plugin_boxed(plugin);
        self
    }

    /// Registers a package into the entry registry.
    pub fn package(mut self, package: impl TesseraPackage) -> Self {
        self.registry.register_package(package);
        self
    }

    /// Overrides the renderer configuration for this entry.
    pub fn config(mut self, config: TesseraConfig) -> Self {
        self.config = config;
        self
    }

    /// Runs the entry point on desktop platforms.
    #[cfg(all(not(target_os = "android"), not(target_family = "wasm")))]
    pub fn run_desktop(self) -> Result<(), winit::error::EventLoopError> {
        #[cfg(feature = "headless")]
        if crate::headless::headless_mode_requested() {
            // Headless mode owns stdout for its JSONL protocol, so tracing is
            // configured there and the window/event-loop path is skipped.
            if let Err(err) =
                crate::headless::run_headless(self.entry, self.registry.finish(), self.config)
            {
                eprintln!("headless session failed: {err}");
                std::process::exit(1);
            }
            return Ok(());
        }

        init_tracing();
        #[cfg(debug_assertions)]
        init_deadlock_detection();
        Renderer::run_with_config(self.entry, self.registry.finish(), self.config)
    }

    /// Runs the entry point on web platforms.
    #[cfg(target_family = "wasm")]
    pub fn run_web(self) -> Result<(), winit::error::EventLoopError> {
        init_web_panic_hook();
        init_tracing();
        Renderer::run_web_with_config(self.entry, self.registry.finish(), self.config)
    }

    /// Runs the entry point on Android.
    #[cfg(target_os = "android")]
    pub fn run_android(self, android_app: AndroidApp) -> Result<(), winit::error::EventLoopError> {
        init_tracing();
        #[cfg(debug_assertions)]
        init_deadlock_detection();
        Renderer::run_with_config(self.entry, self.registry.finish(), android_app, self.config)
    }
}

#[cfg(target_family = "wasm")]
fn init_web_panic_hook() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            web_sys::console::error_1(&info.to_string().into());
            previous(info);
        }));
    });
}

#[cfg(all(debug_assertions, not(target_family = "wasm")))]
fn init_deadlock_detection() {
    use std::{sync::Once, thread, time::Duration};

    static INIT: Once = Once::new();
    INIT.call_once(|| {
        thread::spawn(|| {
            loop {
                thread::sleep(Duration::from_secs(10));
                let deadlocks = parking_lot::deadlock::check_deadlock();
                if deadlocks.is_empty() {
                    continue;
                }

                eprintln!("{} deadlocks detected", deadlocks.len());
                for (idx, threads) in deadlocks.iter().enumerate() {
                    eprintln!("Deadlock #{}", idx);
                    for thread in threads {
                        eprintln!("Thread Id {:#?}", thread.thread_id());
                        eprintln!("{:?}", thread.backtrace());
                    }
                }
            }
        });
    });
}

fn init_tracing() {
    #[cfg(target_family = "wasm")]
    {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_writer(WebConsoleMakeWriter)
            .with_ansi(false)
            .without_time()
            .try_init();
    }

    #[cfg(target_os = "android")]
    {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_max_level(tracing::Level::INFO)
            .try_init();
    }

    #[cfg(all(not(target_os = "android"), not(target_family = "wasm")))]
    {
        let filter = match tracing_subscriber::EnvFilter::try_from_default_env() {
            Ok(filter) => filter,
            Err(_) => match tracing_subscriber::EnvFilter::try_new("error,tessera_ui=info") {
                Ok(filter) => filter,
                Err(_) => tracing_subscriber::EnvFilter::new("error"),
            },
        };

        let _ = tracing_subscriber::fmt()
            .pretty()
            .with_env_filter(filter)
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .try_init();
    }
}

#[cfg(target_family = "wasm")]
struct WebConsoleMakeWriter;

#[cfg(target_family = "wasm")]
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for WebConsoleMakeWriter {
    type Writer = WebConsoleWriter;

    fn make_writer(&'a self) -> Self::Writer {
        WebConsoleWriter {
            level: tracing::Level::INFO,
            buffer: Vec::new(),
        }
    }

    fn make_writer_for(&'a self, metadata: &tracing::Metadata<'_>) -> Self::Writer {
        WebConsoleWriter {
            level: *metadata.level(),
            buffer: Vec::new(),
        }
    }
}

#[cfg(target_family = "wasm")]
struct WebConsoleWriter {
    level: tracing::Level,
    buffer: Vec<u8>,
}

#[cfg(target_family = "wasm")]
impl std::io::Write for WebConsoleWriter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if !self.buffer.is_empty() {
            let message = String::from_utf8_lossy(&self.buffer);
            let message = message.trim_end_matches(['\r', '\n']).into();
            match self.level {
                tracing::Level::ERROR => web_sys::console::error_1(&message),
                tracing::Level::WARN => web_sys::console::warn_1(&message),
                tracing::Level::INFO => web_sys::console::info_1(&message),
                _ => web_sys::console::debug_1(&message),
            }
            self.buffer.clear();
        }
        Ok(())
    }
}

#[cfg(target_family = "wasm")]
impl Drop for WebConsoleWriter {
    fn drop(&mut self) {
        let _ = std::io::Write::flush(self);
    }
}
