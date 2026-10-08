use std::{
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Sender, channel},
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use tessera_build::{AssetBackend, load_tessera_config_from_dir, resolve_assets_dir};

use crate::{color_check, output};

use super::find_package_dir;

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

    // Color-check diagnostics print straight to the terminal, which would
    // corrupt the live view, so they are skipped while `--top` owns the screen.
    if !reporter.is_tui() {
        run_color_check(package, &feature_overrides)?;
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
    if let Some(config) = load_tessera_config_from_dir(&package_dir)?
        && let Some(assets_dir) = resolve_assets_dir(&package_dir, Some(&config))
        && assets_dir.exists()
    {
        watcher.watch(&assets_dir, RecursiveMode::Recursive)?;
    }

    let mut child: Option<Child> = None;
    let mut build_child: Option<Child> = None;
    let mut pending_change = true;
    let mut color_check_current = true;
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
                color_check_current = false;
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
            if !color_check_current && !reporter.is_tui() {
                match run_color_check(package, &feature_overrides) {
                    Ok(()) => {
                        color_check_current = true;
                    }
                    Err(err) => {
                        reporter.error(format!("color check failed: {err}"));
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
                Ok(mut c) => {
                    forward_child_output(&mut c, &reporter);
                    build_child = Some(c);
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
                            Ok(mut c) => {
                                forward_child_output(&mut c, &reporter);
                                child = Some(c);
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

/// Forwards a managed child's stdout/stderr lines to the reporter's log sink.
///
/// The live view owns the terminal, so subprocess output is piped here instead
/// of being inherited; before the view starts these lines are echoed to the
/// console, and afterwards they appear in the view's log pane.
fn forward_child_output(child: &mut Child, reporter: &Reporter) {
    if let Some(stdout) = child.stdout.take() {
        forward_lines(stdout, reporter.clone());
    }
    if let Some(stderr) = child.stderr.take() {
        forward_lines(stderr, reporter.clone());
    }
}

fn forward_lines(stream: impl Read + Send + 'static, reporter: Reporter) {
    let _ = thread::Builder::new()
        .name("tessera-dev-log".to_string())
        .spawn(move || {
            for line in BufReader::new(stream)
                .lines()
                .map_while(std::result::Result::ok)
            {
                reporter.log(line);
            }
        });
}

fn build_tessera_ui_features(
    profiling_output: Option<&Path>,
    profiling_ipc: Option<&str>,
    debug_dirty_overlay: bool,
) -> Vec<String> {
    let mut features = Vec::new();
    if profiling_output.is_some() || profiling_ipc.is_some() {
        features.push("tessera-ui/profiling".to_string());
    }
    if debug_dirty_overlay {
        features.push("tessera-ui/debug-dirty-overlay".to_string());
    }
    features
}

fn configure_tessera_ui_features(
    cmd: &mut Command,
    features: &[String],
    profiling_output: Option<&Path>,
    profiling_ipc: Option<&str>,
) {
    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }
    if let Some(output_path) = profiling_output {
        cmd.env("TESSERA_PROFILING_OUTPUT", output_path);
    }
    if let Some(name) = profiling_ipc {
        cmd.env("TESSERA_PROFILING_IPC", name);
    }
}

fn run_color_check(package: Option<&str>, features: &[String]) -> Result<()> {
    color_check::run(color_check::CheckOptions {
        package,
        target: None,
        target_selection: color_check::TargetSelection::default(),
        features: color_check::FeatureSelection::Selected {
            features,
            no_default_features: false,
        },
        message_format: color_check::MessageFormat::Human,
    })
}

fn resolve_asset_backend(asset_backend_override: Option<AssetBackend>) -> Result<AssetBackend> {
    if let Some(backend) = asset_backend_override {
        if backend == AssetBackend::Platform {
            return Err(anyhow!(
                "Requested platform asset backend is not supported for `cargo tessera dev`; use `cargo tessera android dev` for platform assets"
            ));
        }
        return Ok(backend);
    }

    Ok(AssetBackend::Embed)
}
