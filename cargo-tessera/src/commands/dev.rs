use std::{
    path::PathBuf,
    process::{Child, Command},
    sync::mpsc::channel,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use owo_colors::colored::*;

use super::find_package_dir;

pub fn execute(verbose: bool, package: Option<&str>, release: bool) -> Result<()> {
    println!(
        "{}",
        "Starting development server (auto rebuild/restart)...".bright_cyan()
    );
    if let Some(pkg) = package {
        println!("Package: {}", pkg.bright_yellow());
    }
    println!("{}", "Watching for file changes...".dimmed());

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
                    println!(
                        "\n{}",
                        "Change detected, canceling in-progress build...".bright_yellow()
                    );
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

            println!("\n{}", "Rebuilding project...".bright_yellow());

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

            match build_cmd.spawn() {
                Ok(c) => {
                    build_child = Some(c);
                    pending_change = false;
                }
                Err(e) => {
                    println!("{} Failed to start build: {}", "Error".red(), e);
                }
            }
        }

        // Monitor build progress so we can relaunch the app when ready.
        if let Some(mut active_build) = build_child.take() {
            match active_build.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        println!("{}", "Build failed, waiting for changes...".red());
                    } else if pending_change {
                        println!(
                            "{}",
                            "New changes arrived during build; skipping run and rebuilding..."
                                .yellow()
                        );
                    } else {
                        println!("{}", "Build succeeded, launching app...".green());

                        let mut run_cmd = Command::new("cargo");
                        run_cmd.arg("run");
                        if verbose {
                            run_cmd.arg("-v");
                        }
                        if let Some(pkg) = package {
                            run_cmd.arg("-p").arg(pkg);
                        }

                        match run_cmd.spawn() {
                            Ok(c) => {
                                child = Some(c);
                                println!("{}", "Watching for changes... (Ctrl+C to stop)".cyan());
                            }
                            Err(e) => {
                                println!("{} Failed to start app: {}", "Error".red(), e);
                            }
                        }
                    }
                }
                Ok(None) => {
                    build_child = Some(active_build);
                }
                Err(err) => {
                    println!("{} Failed to check build status: {}", "⚠️".yellow(), err);
                }
            }
        }

        // Monitor the running app so we can exit cleanly if it stops.
        if let Some(mut running_child) = child.take() {
            match running_child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        println!(
                            "\n{}",
                            format!("Application crashed with exit code: {:?}", status.code())
                                .red()
                        );
                    } else {
                        println!("\n{}", "Application exited normally.".green());
                    }
                    println!("{}", "Stopping dev server...".dimmed());
                    break;
                }
                Ok(None) => {
                    child = Some(running_child);
                }
                Err(err) => {
                    println!("{} Failed to check app status: {}", "⚠️".yellow(), err);
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
