//! Application entry builder for registering packages and launching the
//! renderer.
//!
//! ## Usage
//!
//! Configure app startup with packages and plugins before launching the
//! renderer.

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
    session: crate::RuntimeSession,
    entry: Box<dyn Fn()>,
    registry: EntryRegistry,
    config: TesseraConfig,
}

impl EntryPoint {
    /// Returns the runtime owner used by this application entry.
    pub fn runtime_session(&self) -> &crate::RuntimeSession {
        &self.session
    }
    /// Creates a new entry point builder from the root UI function.
    pub fn new<Output>(entry: impl Fn() -> Output + 'static) -> Self {
        let session = crate::RuntimeSession::new();
        session.activate();
        Self {
            session,
            entry: Box::new(move || {
                entry();
            }),
            registry: EntryRegistry::new(),
            config: TesseraConfig::default(),
        }
    }

    /// Adds a render module to the entry registry.
    pub fn module(mut self, module: impl RenderModule + 'static) -> Self {
        let _session = self.session.enter();
        self.registry.add_module(module);
        self
    }

    /// Registers a plugin instance with this application session.
    pub fn plugin(mut self, plugin: impl Plugin) -> Self {
        let _session = self.session.enter();
        self.registry.register_plugin(plugin);
        self
    }

    /// Registers a package into the entry registry.
    pub fn package(mut self, package: impl TesseraPackage) -> Self {
        let _session = self.session.enter();
        self.registry.register_package(package);
        self
    }

    /// Overrides the renderer configuration for this entry.
    pub fn config(mut self, config: TesseraConfig) -> Self {
        self.config = config;
        self
    }

    /// Creates an offscreen renderer with this entry's packages and GPU settings.
    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    pub fn into_headless(
        self,
        mut config: crate::headless::HeadlessConfig,
    ) -> Result<crate::headless::HeadlessRenderer, crate::headless::HeadlessError> {
        let _session = self.session.enter();
        config.sample_count = self.config.sample_count;
        config.max_non_sampler_bindings = self.config.max_non_sampler_bindings;
        crate::headless::HeadlessRenderer::new(self.entry, self.registry.finish(), config)
    }

    /// Runs the entry point on desktop platforms.
    #[cfg(not(any(target_os = "android", target_os = "ios", target_family = "wasm")))]
    pub fn run_desktop(self) -> Result<(), winit::error::EventLoopError> {
        self.run_native()
    }

    /// Runs the entry point on iOS from the application's main thread.
    #[cfg(target_os = "ios")]
    pub fn run_ios(self) -> Result<(), winit::error::EventLoopError> {
        self.run_native()
    }

    #[cfg(all(not(target_os = "android"), not(target_family = "wasm")))]
    fn run_native(self) -> Result<(), winit::error::EventLoopError> {
        let _session = self.session.enter();
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
        let _session = self.session.enter();
        init_web_panic_hook();
        init_tracing();
        Renderer::run_web_with_config(self.entry, self.registry.finish(), self.config)
    }

    /// Runs the entry point on Android.
    #[cfg(target_os = "android")]
    pub fn run_android(self, android_app: AndroidApp) -> Result<(), winit::error::EventLoopError> {
        let _session = self.session.enter();
        init_tracing();
        #[cfg(debug_assertions)]
        init_deadlock_detection();
        Renderer::run_with_config(self.entry, self.registry.finish(), android_app, self.config)
    }
}

#[cfg(target_family = "wasm")]
fn init_web_panic_hook() {
    console_error_panic_hook::set_once();
}

#[cfg(all(debug_assertions, not(target_family = "wasm")))]
struct DeadlockMonitor {
    stop: std::sync::mpsc::Sender<()>,
    worker: Option<std::thread::JoinHandle<()>>,
}

#[cfg(all(debug_assertions, not(target_family = "wasm")))]
impl Default for DeadlockMonitor {
    fn default() -> Self {
        let (stop, receive) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            while matches!(
                receive.recv_timeout(std::time::Duration::from_secs(10)),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            ) {
                let deadlocks = parking_lot::deadlock::check_deadlock();
                if deadlocks.is_empty() {
                    continue;
                }
                eprintln!("{} deadlocks detected", deadlocks.len());
                for (index, threads) in deadlocks.iter().enumerate() {
                    eprintln!("Deadlock #{index}");
                    for thread in threads {
                        eprintln!("Thread Id {:#?}", thread.thread_id());
                        eprintln!("{:?}", thread.backtrace());
                    }
                }
            }
        });
        Self {
            stop,
            worker: Some(worker),
        }
    }
}

#[cfg(all(debug_assertions, not(target_family = "wasm")))]
impl Drop for DeadlockMonitor {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(all(debug_assertions, not(target_family = "wasm")))]
fn init_deadlock_detection() {
    let _ = crate::local_runtime_resource::<DeadlockMonitor>();
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
