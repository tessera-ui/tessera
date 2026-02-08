use std::{
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::mpsc::channel,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use notify::{Event, EventKind, RecursiveMode, Watcher};

use crate::output;

use super::find_package_dir;

pub fn execute(
    verbose: bool,
    package: Option<&str>,
    release: bool,
    profiling_output: Option<&Path>,
) -> Result<()> {
    output::status("Starting", "dev server (auto rebuild/restart)");
    if let Some(pkg) = package {
        output::status("Package", format!("`{}`", pkg));
    }
    if let Some(path) = profiling_output {
        output::status("Profiling", format!("enabled ({})", path.display()));
    }
    output::status("Watching", "for file changes");

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
    for file in ["Cargo.toml", "build.rs"] {
        let path = package_dir.join(file);
        if path.exists() {
            watcher.watch(&path, RecursiveMode::NonRecursive)?;
        }
    }

    let mut child: Option<Child> = None;
    let mut build_child: Option<Child> = None;
    let mut pending_change = true;
    let mut last_change = Instant::now() - Duration::from_secs(1);
    let debounce_window = Duration::from_millis(300);

    loop {
        // Wait for file changes (or time out to check running processes)
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(_) => {
                pending_change = true;
                last_change = Instant::now();

                // Cancel an in-flight build so we only build once per stable tree.
                if let Some(mut active_build) = build_child.take() {
                    output::status("Canceling", "in-progress build due to changes");
                    let _ = active_build.kill();
                    let _ = active_build.wait();
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break,
        }

        // Kick off a build once the tree is quiet and no build is currently running.
        if pending_change && build_child.is_none() && last_change.elapsed() >= debounce_window {
            // Kill previous process
            if let Some(mut c) = child.take() {
                let _ = c.kill();
                let _ = c.wait();
            }

            output::status("Building", "project");

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
            if let Some(path) = profiling_output {
                enable_profiling(&mut build_cmd, path);
            }

            match build_cmd.spawn() {
                Ok(c) => {
                    build_child = Some(c);
                    pending_change = false;
                }
                Err(e) => {
                    output::error(format!("failed to start build: {e}"));
                }
            }
        }

        // Monitor build progress so we can relaunch the app when ready.
        if let Some(mut active_build) = build_child.take() {
            match active_build.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        output::warn("build failed; waiting for changes");
                    } else if pending_change {
                        output::status("Rebuilding", "new changes arrived during build");
                    } else {
                        let mut run_cmd = Command::new("cargo");
                        run_cmd.arg("run");
                        if verbose {
                            run_cmd.arg("-v");
                        }
                        if let Some(pkg) = package {
                            run_cmd.arg("-p").arg(pkg);
                        }
                        if let Some(path) = profiling_output {
                            enable_profiling(&mut run_cmd, path);
                        }

                        match run_cmd.spawn() {
                            Ok(c) => {
                                child = Some(c);
                                output::status("Running", "app (watching for changes)");
                            }
                            Err(e) => {
                                output::error(format!("failed to start app: {e}"));
                            }
                        }
                    }
                }
                Ok(None) => {
                    build_child = Some(active_build);
                }
                Err(err) => {
                    output::warn(format!("failed to check build status: {err}"));
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
                        output::warn(format!("application exited with status {code}"));
                    } else {
                        output::status("Stopped", "application exited normally");
                    }
                    output::status("Stopping", "dev server");
                    break;
                }
                Ok(None) => {
                    child = Some(running_child);
                }
                Err(err) => {
                    output::warn(format!("failed to check app status: {err}"));
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

    Ok(())
}

fn enable_profiling(cmd: &mut Command, output_path: &Path) {
    cmd.arg("--features").arg("tessera-ui/profiling");
    cmd.env("TESSERA_PROFILING_OUTPUT", output_path);
}
