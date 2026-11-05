use std::{
    io::{self, Read},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    process::{Child, ChildStderr, ChildStdout, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, channel},
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use tessera_build::{AssetBackend, load_tessera_config_from_dir, resolve_resource_dirs};

use crate::{call_guard, output};

use super::{build_tessera_ui_features, configure_tessera_ui_features, find_package_dir};

pub fn execute(
    verbose: bool,
    package: Option<&str>,
    release: bool,
    profiling_output: Option<&Path>,
    debug_dirty_overlay: bool,
    asset_backend_override: Option<AssetBackend>,
    top: bool,
) -> Result<()> {
    let asset_backend = resolve_asset_backend(asset_backend_override)?;

    if !top {
        let feature_overrides =
            build_tessera_ui_features(profiling_output, None, debug_dirty_overlay);
        return supervise(SuperviseConfig {
            verbose,
            package: package.map(str::to_string),
            release,
            profiling_output: profiling_output.map(Path::to_path_buf),
            profiling_ipc: None,
            asset_backend,
            feature_overrides,
            reporter: Reporter::Console,
            stop: Arc::new(AtomicBool::new(false)),
        });
    }

    // Live `--top` mode: a private local IPC endpoint is created here and the
    // app connects to it automatically. The endpoint is managed entirely by the
    // tool and is never user-specified.
    let ipc_name = unique_profiler_endpoint();
    let listener = crate::commands::profiling::bind_ipc_listener(&ipc_name)?;
    let feature_overrides =
        build_tessera_ui_features(profiling_output, Some(&ipc_name), debug_dirty_overlay);
    let (logs_tx, logs_rx) = channel();
    let stop = Arc::new(AtomicBool::new(false));

    let package_owned = package.map(str::to_string);
    let profiling_output_owned = profiling_output.map(Path::to_path_buf);
    let ipc_name_owned = ipc_name.clone();
    let supervisor = {
        let stop = Arc::clone(&stop);
        let feature_overrides = feature_overrides.clone();
        thread::spawn(move || {
            supervise(SuperviseConfig {
                verbose,
                package: package_owned,
                release,
                profiling_output: profiling_output_owned,
                profiling_ipc: Some(ipc_name_owned),
                asset_backend,
                feature_overrides,
                reporter: Reporter::Tui(logs_tx),
                stop,
            })
        })
    };

    let options = crate::commands::profiling::WatchOptions {
        refresh: Duration::from_millis(250),
        top: 15,
        window: 120,
        min_count: 1,
    };
    let view = crate::commands::profiling::run_view(listener, options, logs_rx, Arc::clone(&stop));
    // The user quit the live view; tear the dev server down as well.
    stop.store(true, Ordering::Relaxed);
    let _ = supervisor.join();
    view
}

/// Builds a per-process local IPC endpoint name/path for the live profiler.
fn unique_profiler_endpoint() -> String {
    let pid = std::process::id();
    #[cfg(windows)]
    {
        format!("tessera-profiler-{pid}")
    }
    #[cfg(unix)]
    {
        std::env::temp_dir()
            .join(format!("tessera-profiler-{pid}.sock"))
            .to_string_lossy()
            .into_owned()
    }
}

/// Where dev-server status messages go: the terminal, or the live view's log
/// pane.
#[derive(Clone)]
enum Reporter {
    Console,
    Tui(Sender<String>),
}

impl Reporter {
    fn status(&self, label: &str, message: impl AsRef<str>) {
        match self {
            Reporter::Console => output::status(label, message),
            Reporter::Tui(tx) => {
                let _ = tx.send(format!("{label:>10} {}", message.as_ref()));
            }
        }
    }

    fn log(&self, line: impl Into<String>) {
        match self {
            Reporter::Console => println!("{}", line.into()),
            Reporter::Tui(tx) => {
                let _ = tx.send(line.into());
            }
        }
    }

    fn warn(&self, message: impl AsRef<str>) {
        match self {
            Reporter::Console => output::warn(message),
            Reporter::Tui(tx) => {
                let _ = tx.send(format!("{:>10} {}", "warn", message.as_ref()));
            }
        }
    }

    fn error(&self, message: impl AsRef<str>) {
        match self {
            Reporter::Console => output::error(message),
            Reporter::Tui(tx) => {
                let _ = tx.send(format!("{:>10} {}", "error", message.as_ref()));
            }
        }
    }

    fn is_tui(&self) -> bool {
        matches!(self, Reporter::Tui(_))
    }
}

struct SuperviseConfig {
    verbose: bool,
    package: Option<String>,
    release: bool,
    profiling_output: Option<PathBuf>,
    profiling_ipc: Option<String>,
    asset_backend: AssetBackend,
    feature_overrides: Vec<String>,
    reporter: Reporter,
    stop: Arc<AtomicBool>,
}

fn supervise(config: SuperviseConfig) -> Result<()> {
    let SuperviseConfig {
        verbose,
        package,
        release,
        profiling_output,
        profiling_ipc,
        asset_backend,
        feature_overrides,
        reporter,
        stop,
    } = config;
    let package = package.as_deref();
    let profiling_output = profiling_output.as_deref();
    let profiling_ipc = profiling_ipc.as_deref();

    // Call-check diagnostics print straight to the terminal, which would
    // corrupt the live view, so they are skipped while `--top` owns the screen.
    if !reporter.is_tui() {
        run_call_guard(package, &feature_overrides, release)?;
    }

    reporter.status("Starting", "dev server (auto rebuild/restart)");
    if let Some(pkg) = package {
        reporter.status("Package", format!("`{pkg}`"));
    }
    if let Some(path) = profiling_output {
        reporter.status("Profiling", format!("enabled ({})", path.display()));
    }
    if let Some(name) = profiling_ipc {
        reporter.status("Profiling", format!("live view over local IPC ({name})"));
    }
    reporter.status("Watching", "for file changes");

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res
            && matches!(
                event.kind,
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
            )
        {
            let _ = tx.send(());
        }
    })?;

    // Determine the package directory to watch
    let package_dir = if let Some(pkg) = package {
        find_package_dir(pkg)?
    } else {
        PathBuf::from(".")
    };
    reporter.status("Assets", format!("backend `{}`", asset_backend.as_str()));

    // Watch the src directory
    let src_path = package_dir.join("src");
    if src_path.exists() {
        watcher.watch(&src_path, RecursiveMode::Recursive)?;
    } else {
        return Err(anyhow!(
            "Source directory not found: {}",
            src_path.display()
        ));
    }

    // Watch Cargo.toml and build.rs in the package directory
    for file in [
        "Cargo.toml",
        "build.rs",
        "tessera-app.toml",
        "tessera-config.toml",
    ] {
        let path = package_dir.join(file);
        if path.exists() {
            watcher.watch(&path, RecursiveMode::NonRecursive)?;
        }
    }
    let config = load_tessera_config_from_dir(&package_dir)?;
    for directory in resolve_resource_dirs(&package_dir, config.as_ref()) {
        if directory.is_dir() {
            watcher.watch(&directory, RecursiveMode::Recursive)?;
        }
    }

    let mut child: Option<ManagedChild> = None;
    let mut build_child: Option<ManagedChild> = None;
    let mut pending_change = true;
    let mut call_guard_current = true;
    let mut last_change = Instant::now() - Duration::from_secs(1);
    let debounce_window = Duration::from_millis(300);

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        // Wait for file changes (or time out to check running processes)
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(_) => {
                pending_change = true;
                call_guard_current = false;
                last_change = Instant::now();

                // Cancel an in-flight build so we only build once per stable
                // tree.
                if let Some(mut active_build) = build_child.take() {
                    reporter.status("Canceling", "in-progress build due to changes");
                    let _ = active_build.kill();
                    let _ = active_build.wait();
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break,
        }

        // Kick off a build once the tree is quiet and no build is currently
        // running.
        if pending_change && build_child.is_none() && last_change.elapsed() >= debounce_window {
            if !call_guard_current && !reporter.is_tui() {
                match run_call_guard(package, &feature_overrides, release) {
                    Ok(()) => {
                        call_guard_current = true;
                    }
                    Err(err) => {
                        reporter.error(format!("Tessera call checking failed: {err}"));
                        reporter.warn("waiting for changes");
                        pending_change = false;
                        continue;
                    }
                }
            }

            // Kill previous process
            if let Some(mut c) = child.take() {
                let _ = c.kill();
                let _ = c.wait();
            }

            reporter.status("Building", "project");

            let mut build_cmd = Command::new("cargo");
            build_cmd.arg("build");
            if release {
                build_cmd.arg("--release");
            }
            if verbose {
                build_cmd.arg("-v");
            }
            if let Some(pkg) = package {
                build_cmd.arg("-p").arg(pkg);
            }
            configure_tessera_ui_features(
                &mut build_cmd,
                &feature_overrides,
                profiling_output,
                profiling_ipc,
            );
            build_cmd.env("TESSERA_ASSET_BACKEND", asset_backend.as_str());
            if reporter.is_tui() {
                build_cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
            }

            match build_cmd.spawn() {
                Ok(c) => {
                    build_child = Some(ManagedChild::new(c, &reporter));
                    pending_change = false;
                }
                Err(e) => {
                    reporter.error(format!("failed to start build: {e}"));
                }
            }
        }

        // Monitor build progress so we can relaunch the app when ready.
        if let Some(mut active_build) = build_child.take() {
            match active_build.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        reporter.warn("build failed; waiting for changes");
                    } else if pending_change {
                        reporter.status("Rebuilding", "new changes arrived during build");
                    } else {
                        let mut run_cmd = Command::new("cargo");
                        run_cmd.arg("run");
                        if verbose {
                            run_cmd.arg("-v");
                        }
                        if let Some(pkg) = package {
                            run_cmd.arg("-p").arg(pkg);
                        }
                        configure_tessera_ui_features(
                            &mut run_cmd,
                            &feature_overrides,
                            profiling_output,
                            profiling_ipc,
                        );
                        run_cmd.env("TESSERA_ASSET_BACKEND", asset_backend.as_str());
                        if reporter.is_tui() {
                            run_cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
                        }

                        match run_cmd.spawn() {
                            Ok(c) => {
                                child = Some(ManagedChild::new(c, &reporter));
                                reporter.status("Running", "app (watching for changes)");
                            }
                            Err(e) => {
                                reporter.error(format!("failed to start app: {e}"));
                            }
                        }
                    }
                }
                Ok(None) => {
                    build_child = Some(active_build);
                }
                Err(err) => {
                    reporter.warn(format!("failed to check build status: {err}"));
                }
            }
        }

        // Monitor the running app so we can exit cleanly if it stops.
        if let Some(mut running_child) = child.take() {
            match running_child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        let code = status
                            .code()
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        reporter.warn(format!("application exited with status {code}"));
                    } else {
                        reporter.status("Stopped", "application exited normally");
                    }
                    reporter.status("Stopping", "dev server");
                    break;
                }
                Ok(None) => {
                    child = Some(running_child);
                }
                Err(err) => {
                    reporter.warn(format!("failed to check app status: {err}"));
                }
            }
        }
    }

    // Cleanup
    if let Some(mut build) = build_child {
        let _ = build.kill();
        let _ = build.wait();
    }
    if let Some(mut c) = child {
        let _ = c.kill();
        let _ = c.wait();
    }

    stop.store(true, Ordering::Relaxed);
    Ok(())
}

struct ManagedChild {
    child: Child,
    readers: Vec<OutputReader>,
}

impl ManagedChild {
    fn new(mut child: Child, reporter: &Reporter) -> Self {
        let mut readers = Vec::new();
        if let Some(stdout) = child.stdout.take() {
            match OutputReader::new(stdout, reporter.clone()) {
                Ok(reader) => readers.push(reader),
                Err(error) => reporter.warn(format!("failed to read child stdout: {error}")),
            }
        }
        if let Some(stderr) = child.stderr.take() {
            match OutputReader::new(stderr, reporter.clone()) {
                Ok(reader) => readers.push(reader),
                Err(error) => reporter.warn(format!("failed to read child stderr: {error}")),
            }
        }
        Self { child, readers }
    }
}

impl Deref for ManagedChild {
    type Target = Child;

    fn deref(&self) -> &Child {
        &self.child
    }
}

impl DerefMut for ManagedChild {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.child
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        if !matches!(self.child.try_wait(), Ok(Some(_))) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        self.readers.clear();
    }
}

trait OutputPipe: Read + Send + 'static {
    fn ready(&self) -> io::Result<bool>;
}

macro_rules! output_pipe {
    ($pipe:ty) => {
        impl OutputPipe for $pipe {
            fn ready(&self) -> io::Result<bool> {
                pipe_ready(self)
            }
        }
    };
}

output_pipe!(ChildStdout);
output_pipe!(ChildStderr);

#[cfg(windows)]
fn pipe_ready(pipe: &impl std::os::windows::io::AsRawHandle) -> io::Result<bool> {
    let mut available = 0;
    // Child output handles remain owned by the reader for this call.
    let success = unsafe {
        windows_sys::Win32::System::Pipes::PeekNamedPipe(
            pipe.as_raw_handle(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            &mut available,
            std::ptr::null_mut(),
        )
    };
    if success == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(available > 0)
}

#[cfg(unix)]
fn pipe_ready(pipe: &impl std::os::fd::AsRawFd) -> io::Result<bool> {
    let mut descriptor = libc::pollfd {
        fd: pipe.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    // The descriptor remains owned by the reader and the array has one entry.
    let ready = unsafe { libc::poll(&mut descriptor, 1, 0) };
    if ready < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(ready > 0)
}

struct OutputReader {
    stop: Sender<()>,
    worker: Option<thread::JoinHandle<()>>,
}

impl OutputReader {
    fn new(stream: impl OutputPipe, reporter: Reporter) -> io::Result<Self> {
        let (stop, stopped) = channel();
        let worker = thread::Builder::new()
            .name("tessera-dev-log".to_string())
            .spawn(move || read_output(stream, reporter, stopped))?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
}

impl Drop for OutputReader {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn read_output(mut stream: impl OutputPipe, reporter: Reporter, stopped: Receiver<()>) {
    let mut pending = Vec::new();
    let mut chunk = [0; 4096];
    loop {
        match stream.ready() {
            Ok(true) => match stream.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(count) => {
                    pending.extend_from_slice(&chunk[..count]);
                    while let Some(end) = pending.iter().position(|byte| *byte == b'\n') {
                        let mut line: Vec<_> = pending.drain(..=end).collect();
                        line.pop();
                        if line.last() == Some(&b'\r') {
                            line.pop();
                        }
                        reporter.log(String::from_utf8_lossy(&line));
                    }
                }
            },
            Ok(false) => match stopped.recv_timeout(Duration::from_millis(10)) {
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                _ => break,
            },
            Err(_) => break,
        }
        if stopped.try_recv().is_ok() {
            break;
        }
    }
    if !pending.is_empty() {
        reporter.log(String::from_utf8_lossy(&pending));
    }
}

fn run_call_guard(package: Option<&str>, features: &[String], release: bool) -> Result<()> {
    call_guard::run(call_guard::CheckOptions {
        features: call_guard::FeatureSelection::Selected {
            features,
            no_default_features: false,
        },
        release,
        ..call_guard::CheckOptions::new(package, None)
    })
}

fn resolve_asset_backend(asset_backend_override: Option<AssetBackend>) -> Result<AssetBackend> {
    if let Some(backend) = asset_backend_override {
        if backend == AssetBackend::Platform {
            return Err(anyhow!(
                "Requested platform asset backend is not supported for `cargo tessera dev`; use `cargo tessera build --asset-backend platform` or `cargo tessera android dev` for platform assets"
            ));
        }
        return Ok(backend);
    }

    Ok(AssetBackend::Embed)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn managed_child_cancels_idle_readers_and_terminates_before_closing_input() {
        const CHILD: &str = "TESSERA_DEV_LOG_CHILD";
        if let Some(directory) = std::env::var_os(CHILD) {
            println!("owned child output");
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            std::fs::write(
                PathBuf::from(directory).join("input-closed"),
                b"unexpected EOF",
            )
            .unwrap();
            return;
        }
        let directory = tempfile::tempdir().unwrap();
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "commands::dev::tests::managed_child_cancels_idle_readers_and_terminates_before_closing_input", "--nocapture"])
            .env(CHILD, directory.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn().unwrap();
        let (logs, output) = channel();
        let reporter = Reporter::Tui(logs);
        let mut owner = ManagedChild::new(child, &reporter);
        drop(reporter);
        loop {
            let line = output
                .recv_timeout(Duration::from_secs(15))
                .expect("child output must reach the live view");
            if line.contains("owned child output") {
                break;
            }
        }
        owner.readers.clear();
        while output.recv_timeout(Duration::from_secs(15)).is_ok() {}
        assert!(
            matches!(
                output.try_recv(),
                Err(std::sync::mpsc::TryRecvError::Disconnected)
            ),
            "reader shutdown must release the output channel"
        );
        assert!(
            owner.try_wait().unwrap().is_none(),
            "an idle log reader must stop while its pipe writer is still alive"
        );
        drop(owner);
        assert!(
            !directory.path().join("input-closed").exists(),
            "child teardown must terminate the process before dropping its input"
        );
    }
}
